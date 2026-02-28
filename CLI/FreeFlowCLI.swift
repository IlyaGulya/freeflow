import Foundation

let notifications: [String: String] = [
    "start": "com.freeflow.start-recording",
    "stop": "com.freeflow.stop-recording",
    "toggle": "com.freeflow.toggle-recording",
]

func printUsage() {
    let name = (CommandLine.arguments.first as NSString?)?.lastPathComponent ?? "freeflow"
    fputs("""
    Usage: \(name) <command>

    Commands:
      start    Start recording
      stop     Stop recording and transcribe
      toggle   Toggle recording on/off
      status   Print current state (recording/idle)

    """, stderr)
}

func handleStatus() {
    let center = DistributedNotificationCenter.default()
    var receivedResponse = false

    let observer = center.addObserver(
        forName: .init("com.freeflow.status-response"),
        object: nil,
        queue: nil
    ) { notification in
        let state = notification.object as? String ?? "unknown"
        print(state)
        receivedResponse = true
        CFRunLoopStop(CFRunLoopGetMain())
    }

    center.postNotificationName(
        .init("com.freeflow.status-request"),
        object: nil,
        userInfo: nil,
        deliverImmediately: true
    )

    CFRunLoopRunInMode(.defaultMode, 2.0, false)
    center.removeObserver(observer)

    if !receivedResponse {
        fputs("No response from FreeFlow (is it running?)\n", stderr)
        exit(1)
    }
}

guard CommandLine.arguments.count == 2 else {
    printUsage()
    exit(1)
}

let command = CommandLine.arguments[1]

if command == "status" {
    handleStatus()
} else if let notificationName = notifications[command] {
    let center = DistributedNotificationCenter.default()
    var received = false

    // Subscribe to ack BEFORE sending the command to avoid race
    let observer = center.addObserver(
        forName: .init("com.freeflow.ack"),
        object: nil,
        queue: nil
    ) { notification in
        let payload = notification.object as? String ?? ""
        if payload.hasPrefix("\(command):") {
            let state = String(payload.dropFirst(command.count + 1))
            print("\(command): ok (state: \(state))")
            received = true
            CFRunLoopStop(CFRunLoopGetMain())
        }
    }

    center.postNotificationName(
        .init(notificationName),
        object: nil,
        userInfo: nil,
        deliverImmediately: true
    )

    CFRunLoopRunInMode(.defaultMode, 2.0, false)
    center.removeObserver(observer)

    if !received {
        fputs("No response from FreeFlow (is it running?)\n", stderr)
        exit(1)
    }
} else {
    fputs("Unknown command: \(command)\n", stderr)
    printUsage()
    exit(1)
}
