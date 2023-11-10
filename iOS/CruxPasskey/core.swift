import AuthenticationServices
import Foundation
import SharedTypes
import SwiftUI

@MainActor
class Core: ObservableObject {
    @Published var view: ViewModel

    private var passkeyCapability = PasskeyCapability()

    init() {
        view = try! .bincodeDeserialize(input: [UInt8](CruxPasskey.view()))
    }

    func update(_ event: Event) {
        let effects = [UInt8](processEvent(Data(try! event.bincodeSerialize())))

        let requests: [Request] = try! .bincodeDeserialize(input: effects)
        for request in requests {
            processEffect(request)
        }
    }

    func processEffect(_ request: Request) {
        switch request.effect {
        case let .http(req):
            Task {
                let response = try! await requestHttp(req).get()

                let effects = [UInt8](handleResponse(Data(request.uuid), Data(try! response.bincodeSerialize())))

                let requests: [Request] = try! .bincodeDeserialize(input: effects)
                for request in requests {
                    processEffect(request)
                }
            }
        case let .passkey(req):
            Task {
                let response = try! await passkeyCapability.request(req).get()

                let effects = [UInt8](handleResponse(Data(request.uuid), Data(try! response.bincodeSerialize())))

                let requests: [Request] = try! .bincodeDeserialize(input: effects)
                for request in requests {
                    processEffect(request)
                }
            }
        case .render:
            view = try! .bincodeDeserialize(input: [UInt8](CruxPasskey.view()))
        }
    }
}
