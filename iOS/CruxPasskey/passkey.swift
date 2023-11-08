import AuthenticationServices
import SharedTypes
import SwiftUI

enum PasskeyError: Error {
    case generic(Error)
    case message(String)
}

class PasskeyController:
    NSObject,
    ASAuthorizationControllerPresentationContextProviding,
    ASAuthorizationControllerDelegate
{
    let domain = "crux-passkey-server-yrx9iojr.fermyon.app"

    func presentationAnchor(for controller: ASAuthorizationController) -> ASPresentationAnchor {
        return ASPresentationAnchor()
    }

    func authorizationController(controller: ASAuthorizationController, didCompleteWithError error: Error) {
        print("authorization failed: \(error)")
    }

    func authorizationController(controller: ASAuthorizationController, didCompleteWithAuthorization authorization: ASAuthorization) {
        switch authorization.credential {
        case let credential as ASAuthorizationPlatformPublicKeyCredentialRegistration:
            print("A new credential was registered: \(credential)")
            Task.init {
                try! await self.registerFinish(credentialRegistration: credential)
            }
        case let credential as ASAuthorizationPlatformPublicKeyCredentialAssertion:
            print("A credential was used to authenticate: \(credential)")
            Task.init {
                try! await self.loginFinish(credentialAssertion: credential)
            }
        default:
            print("unknown authentication method")
        }
    }

    func requestPasskey(_ request: PasskeyOperation) async -> Result<PasskeyOutput, PasskeyError> {
        switch request {
        case .register(let userName):
            await self.signUpWith(userName: userName)
            return .success(.registered)
        case .login(let userName):
            await self.signInWith(userName: userName)
            return .success(.loggedIn)
        }
    }

    fileprivate func registerStart(userName: String) async throws -> PublicKey {
        let req = URLRequest(url: URL(string: "https://\(domain)/auth/register_start/\(userName)")!)
        let (data, response) = try! await URLSession.shared.data(for: req)
        let httpResponse = response as? HTTPURLResponse

        let status = UInt16(httpResponse!.statusCode)
        guard status >= 200 && status < 300 else {
            let body = String(decoding: data, as: UTF8.self)
            let msg = "register start, HTTP status: \(status), body: \(body)"
            print(msg)
            throw PasskeyError.message(msg)
        }

        let decoded = try! JSONDecoder().decode(PublicKeyResponse.self, from: data)
        return decoded.publicKey
    }

    private func registerFinish(credentialRegistration: ASAuthorizationPlatformPublicKeyCredentialRegistration) async throws {
        let id = credentialRegistration.credentialID.base64EncodedString()
        let payload = RegisterCredential(
            id: id,
            rawId: id,
            type: "public-key",
            response: AttestationResponse(
                attestationObject: credentialRegistration.rawAttestationObject!.base64EncodedString(),
                clientDataJSON: credentialRegistration.rawClientDataJSON.base64EncodedString()
            )
        )
        let body = try! JSONEncoder().encode(payload)
        print(String(decoding: body, as: UTF8.self))
        var req = URLRequest(url: URL(string: "https://\(domain)/auth/register_finish")!)
        req.httpMethod = "POST"
        req.httpBody = body
        req.setValue("application/json", forHTTPHeaderField: "Content-Type")

        let (data, response) = try! await URLSession.shared.data(for: req)
        let httpResponse = response as? HTTPURLResponse

        let status = UInt16(httpResponse!.statusCode)
        guard status >= 200 && status < 300 else {
            let body = String(decoding: data, as: UTF8.self)
            let msg = "register finish, HTTP status: \(status), body: \(body)"
            print(msg)
            throw PasskeyError.message(msg)
        }
    }

    private func loginStart(userName: String) async throws -> PublicKey {
        let req = URLRequest(url: URL(string: "https://\(domain)/auth/login_start/\(userName)")!)
        let (data, response) = try! await URLSession.shared.data(for: req)
        let httpResponse = response as? HTTPURLResponse

        let status = UInt16(httpResponse!.statusCode)
        guard status >= 200 && status < 300 else {
            let body = String(decoding: data, as: UTF8.self)
            let msg = "login start, HTTP status: \(status), body: \(body)"
            print(msg)
            throw PasskeyError.message(msg)
        }

        let decoded = try! JSONDecoder().decode(PublicKeyResponse.self, from: data)
        return decoded.publicKey
    }

    private func loginFinish(credentialAssertion: ASAuthorizationPlatformPublicKeyCredentialAssertion) async throws {
        let id = credentialAssertion.credentialID.base64EncodedString()
        let payload = Credential(
            id: id,
            rawId: id,
            type: "public-key",
            response: AssertionResponse(
                authenticatorData: credentialAssertion.rawAuthenticatorData.base64EncodedString(),
                clientDataJSON: credentialAssertion.rawClientDataJSON.base64EncodedString(),
                signature: credentialAssertion.signature.base64EncodedString(),
                userHandle: credentialAssertion.userID.base64EncodedString()
            )
        )

        var req = URLRequest(url: URL(string: "https://\(domain)/auth/login_finish")!)
        req.httpMethod = "POST"
        req.httpBody = try! JSONEncoder().encode(payload)
        req.setValue("application/json", forHTTPHeaderField: "Content-Type")

        let (data, response) = try! await URLSession.shared.data(for: req)
        let httpResponse = response as? HTTPURLResponse

        let status = UInt16(httpResponse!.statusCode)
        guard status >= 200 && status < 300 else {
            let body = String(decoding: data, as: UTF8.self)
            let msg = "login finish, HTTP status: \(status), body: \(body)"
            print(msg)
            throw PasskeyError.message(msg)
        }
    }

    func signUpWith(userName: String) async {
        let provider = ASAuthorizationPlatformPublicKeyCredentialProvider(relyingPartyIdentifier: domain)

        let publicKey = try! await registerStart(userName: userName)
        let decodedChallenge = Data(base64URLEncoded: publicKey.challenge)
        let decodedUserId = Data(base64URLEncoded: publicKey.user!.id)

        let request = provider.createCredentialRegistrationRequest(challenge: decodedChallenge!, name: userName, userID: decodedUserId!)

        let controller = ASAuthorizationController(authorizationRequests: [request])
        controller.delegate = self
        controller.presentationContextProvider = self
        controller.performRequests()
    }

    func signInWith(userName: String) async {
        let provider = ASAuthorizationPlatformPublicKeyCredentialProvider(relyingPartyIdentifier: domain)

        let publicKey = try! await loginStart(userName: userName)
        let decodedChallenge = Data(base64URLEncoded: publicKey.challenge)
        let request = provider.createCredentialAssertionRequest(challenge: decodedChallenge!)
        let controller = ASAuthorizationController(authorizationRequests: [request])
        controller.delegate = self
        controller.presentationContextProvider = self
        controller.performRequests()
    }
}

private struct PublicKeyResponse: Codable {
    var publicKey: PublicKey
}

private struct PublicKey: Codable {
    var user: User?
    var challenge: String
}

private struct User: Codable {
    var id: String
    var name: String
    var displayName: String
}

private struct RegisterCredential: Codable {
    var id: String
    var rawId: String
    var type: String
    var response: AttestationResponse
}

private struct AttestationResponse: Codable {
    var attestationObject: String
    var clientDataJSON: String
}

private struct Credential: Codable {
    var id: String
    var rawId: String
    var type: String
    var response: AssertionResponse
}

private struct AssertionResponse: Codable {
    var authenticatorData: String
    var clientDataJSON: String
    var signature: String
    var userHandle: String
}
