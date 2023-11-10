import SharedTypes
import SwiftUI
import AuthenticationServices

struct ContentView: View {
    @ObservedObject var core: Core
    @State private var username: String = ""
    @FocusState private var usernameFieldIsFocused: Bool
    
    init(core: Core) {
        self.core = core
        core.update(.serverUrl("https://\(rp_id!)"))
    }
    
    var body: some View {
        Section {
            LabeledContent("User name") {
                TextField(
                    "User name",
                    text: $username,
                    prompt: Text("Required")
                )
                .textContentType(.username)
                .keyboardType(.emailAddress)
                .onChange(of: username) { core.update(.validate(username)) }
                .focused($usernameFieldIsFocused)
                .textInputAutocapitalization(.never)
                .disableAutocorrection(true)
                .textFieldStyle(.roundedBorder)
                .padding(5)
                .shadow(color: .gray, radius: 2)
            }.padding(10)
            HStack {
                ActionButton(label: "Sign in", color: .green) {
                    core.update(.login(username))
                }
                ActionButton(label: "Sign up", color: .yellow) {
                    core.update(.register(username))
                }
            }
            switch core.view.status {
            case .none: Text(" ")
            case let .error(msg): Text(msg)
            case let .info(msg): Text(msg)
            }
        } header: {
            Image(systemName: "person.badge.key.fill")
                .font(.system(.largeTitle, design: .rounded))
                .fontWeight(.semibold)
                .foregroundColor(.white)
                .frame(width: 60, height: 60)
                .background(Color.accentColor.gradient, in: Circle())
            Text("Sign in, or sign up to create an account")
        } footer: {
            Label("""
                When you sign up with a passkey, all you need is a user name. \
                The passkey will be available on all of your devices.
                """, systemImage: "person.badge.key.fill")
        }
        .padding(10)
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
