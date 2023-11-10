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
        case .createCredential(let bytes):
            let challenge = try! JSONDecoder().decode(PublicKeyResponse.self, from: Data(bytes))
            let controller = PasskeyController(pubKeyResponse: challenge)
            let credential = await controller.signUp() as! ASAuthorizationPlatformPublicKeyCredentialRegistration
            let id = credential.credentialID.base64EncodedString()
            let payload = RegisterCredential(
                id: id,
                rawId: id,
                type: "public-key",
                response: AttestationResponse(
                    attestationObject: credential.rawAttestationObject!.base64EncodedString(),
                    clientDataJSON: credential.rawClientDataJSON.base64EncodedString()
                )
            )
            let body = try! JSONEncoder().encode(payload)
            return .success(.registerCredential([UInt8](body)))
        case .requestCredential(let bytes):
            let challenge = try! JSONDecoder().decode(PublicKeyResponse.self, from: Data(bytes))
            let controller = PasskeyController(pubKeyResponse: challenge)
            let credential = await controller.signIn() as! ASAuthorizationPlatformPublicKeyCredentialAssertion
            let id = credential.credentialID.base64EncodedString()
            let payload = Credential(
                id: id,
                rawId: id,
                type: "public-key",
                response: AssertionResponse(
                    authenticatorData: credential.rawAuthenticatorData.base64EncodedString(),
                    clientDataJSON: credential.rawClientDataJSON.base64EncodedString(),
                    signature: credential.signature.base64EncodedString(),
                    userHandle: credential.userID.base64EncodedString()
                )
            )
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
    let domain = "crux-passkey-server-9uqexpm2.fermyon.app"
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
    
    func presentationAnchor(for controller: ASAuthorizationController) -> ASPresentationAnchor {
        return ASPresentationAnchor()
    }

    func authorizationController(controller: ASAuthorizationController, didCompleteWithError error: Error) {
        print("authorization failed: \(error)")
    }

    func authorizationController(controller: ASAuthorizationController, didCompleteWithAuthorization authorization: ASAuthorization) {
        guard let completion else {
            return
        }
        completion(authorization.credential)
    }

    func signUp(with completion: @escaping (ASAuthorizationCredential) -> Void) {
        self.completion = completion
        
        let provider = ASAuthorizationPlatformPublicKeyCredentialProvider(relyingPartyIdentifier: domain)

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
        
        let provider = ASAuthorizationPlatformPublicKeyCredentialProvider(relyingPartyIdentifier: domain)

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
