import Foundation
import SharedTypes

@MainActor
class Core: ObservableObject {
    @Published var view: ViewModel

    init() {
        self.view = try! .bincodeDeserialize(input: [UInt8](CruxPasskey.view()))
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
        case .render:
            view = try! .bincodeDeserialize(input: [UInt8](CruxPasskey.view()))
            
        case let .passkey(req):
            Task {
                let response = try! await requestPasskey(req).get()
                
                let effects = [UInt8](handleResponse(Data(request.uuid), Data(try! response.bincodeSerialize())))
                
                let requests: [Request] = try! .bincodeDeserialize(input: effects)
                for request in requests {
                    processEffect(request)
                }
            }
        }
    }
}
