import SharedTypes
import SwiftUI

struct ContentView: View {
    @ObservedObject var core: Core
    @State private var username: String = ""
    @FocusState private var usernameFieldIsFocused: Bool

    var body: some View {
        VStack {
            Section(header: Text("User name")) {
                TextField(
                    "User name",
                    text: $username,
                    prompt: Text("Required")
                )
                .onChange(of: username) { core.update(.validate(username))}
                .focused($usernameFieldIsFocused)
                .textInputAutocapitalization(.never)
                .disableAutocorrection(true)
                .textFieldStyle(.roundedBorder)
                .padding(10)
                .shadow(color: .gray, radius: 2)
            }
            HStack {
                ActionButton(label: "Register", color: .yellow) {
                    core.update(.register(username))
                }
                ActionButton(label: "Login", color: .green) {
                    core.update(.login(username))
                }
            }
            switch core.view.status {
            case .none: Text(" ")
            case let .error(msg): Text(msg)
            case let .info(msg): Text(msg)
            }
        }
    }
}

struct ActionButton: View {
    var label: String
    var color: Color
    var action: () -> Void

    init(label: String, color: Color, action: @escaping () -> Void) {
        self.label = label
        self.color = color
        self.action = action
    }

    var body: some View {
        Button(action: action) {
            Text(label)
                .fontWeight(.bold)
                .font(.body)
                .padding(EdgeInsets(top: 10, leading: 15, bottom: 10, trailing: 15))
                .background(color)
                .cornerRadius(10)
                .foregroundColor(.white)
                .padding()
        }
    }
}

struct ContentView_Previews: PreviewProvider {
    static var previews: some View {
        ContentView(core: Core())
    }
}
