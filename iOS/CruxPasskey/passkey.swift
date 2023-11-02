import SharedTypes
import SwiftUI

enum PasskeyError: Error {
    case generic(Error)
    case message(String)
}

func requestPasskey(_ request: PasskeyOperation) async -> Result<PasskeyOutput, PasskeyError> {
    return switch request {
    case .register: .success(.registered);
    case .login: .success(.loggedIn);
    }
}
