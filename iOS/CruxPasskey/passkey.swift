import AuthenticationServices
import SharedTypes
import SwiftUI

enum PasskeyError: Error {
    case generic(Error)
    case message(String)
}

class PasskeyCapability {
    func request(_ request: PasskeyOperation) async -> Result<PasskeyOutput, PasskeyError> {
        switch request {
        case let .createCredential(bytes):
            let challenge = try! PublicKeyResponse.from(bytes)
            let controller = PasskeyController(pubKeyResponse: challenge)

            let credential = await controller.signUp() as! ASAuthorizationPlatformPublicKeyCredentialRegistration

            let payload = RegisterCredential.from(credential)
            let body = try! JSONEncoder().encode(payload)

            return .success(.registerCredential([UInt8](body)))
        case let .requestCredential(bytes):
            let challenge = try! PublicKeyResponse.from(bytes)
            let controller = PasskeyController(pubKeyResponse: challenge)

            let credential = await controller.signIn() as! ASAuthorizationPlatformPublicKeyCredentialAssertion

            let payload = Credential.from(credential)
            let body = try! JSONEncoder().encode(payload)

            return .success(.credential([UInt8](body)))
        }
    }
}

class PasskeyController:
    NSObject,
    ASAuthorizationControllerPresentationContextProviding,
    ASAuthorizationControllerDelegate
{
    var completion: ((ASAuthorizationCredential) -> Void)?

    fileprivate var pubKeyResponse: PublicKeyResponse

    fileprivate init(pubKeyResponse: PublicKeyResponse) {
        self.pubKeyResponse = pubKeyResponse
    }

    func signUp() async -> ASAuthorizationCredential {
        await withCheckedContinuation { continuation in
            signUp(with: { credential in
                continuation.resume(returning: credential)
            })
        }
    }

    func signIn() async -> ASAuthorizationCredential {
        await withCheckedContinuation { continuation in
            signIn(with: { credential in
                continuation.resume(returning: credential)
            })
        }
    }

    func presentationAnchor(for _: ASAuthorizationController) -> ASPresentationAnchor {
        return ASPresentationAnchor()
    }

    func authorizationController(controller _: ASAuthorizationController, didCompleteWithError error: Error) {
        print("authorization failed: \(error)")
    }

    func authorizationController(controller _: ASAuthorizationController, didCompleteWithAuthorization authorization: ASAuthorization) {
        guard let completion else {
            return
        }
        completion(authorization.credential)
    }

    func signUp(with completion: @escaping (ASAuthorizationCredential) -> Void) {
        self.completion = completion

        let provider = ASAuthorizationPlatformPublicKeyCredentialProvider(relyingPartyIdentifier: rp_id!)

        let publicKey = pubKeyResponse.publicKey
        let decodedChallenge = Data(base64URLEncoded: publicKey.challenge)
        let decodedUserId = Data(base64URLEncoded: publicKey.user!.id)

        let request = provider.createCredentialRegistrationRequest(challenge: decodedChallenge!, name: pubKeyResponse.publicKey.user!.name, userID: decodedUserId!)

        let controller = ASAuthorizationController(authorizationRequests: [request])
        controller.delegate = self
        controller.presentationContextProvider = self
        controller.performRequests()
    }

    func signIn(with completion: @escaping (ASAuthorizationCredential) -> Void) {
        self.completion = completion

        let provider = ASAuthorizationPlatformPublicKeyCredentialProvider(relyingPartyIdentifier: rp_id!)

        let publicKey = pubKeyResponse.publicKey
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

    static func from(_ of: [UInt8]) throws -> Self {
        return try JSONDecoder().decode(PublicKeyResponse.self, from: Data(of))
    }
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

    static func from(_ of: ASAuthorizationPlatformPublicKeyCredentialRegistration) -> Self {
        let id = of.credentialID.base64EncodedString()
        return RegisterCredential(
            id: id,
            rawId: id,
            type: "public-key",
            response: AttestationResponse(
                attestationObject: of.rawAttestationObject!.base64EncodedString(),
                clientDataJSON: of.rawClientDataJSON.base64EncodedString()
            )
        )
    }
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

    static func from(_ of: ASAuthorizationPlatformPublicKeyCredentialAssertion) -> Self {
        let id = of.credentialID.base64EncodedString()
        return Credential(
            id: id,
            rawId: id,
            type: "public-key",
            response: AssertionResponse(
                authenticatorData: of.rawAuthenticatorData.base64EncodedString(),
                clientDataJSON: of.rawClientDataJSON.base64EncodedString(),
                signature: of.signature.base64EncodedString(),
                userHandle: of.userID.base64EncodedString()
            )
        )
    }
}

private struct AssertionResponse: Codable {
    var authenticatorData: String
    var clientDataJSON: String
    var signature: String
    var userHandle: String
}
