import ExpoModulesCore

public class NozyFfiModule: Module {
  public func definition() -> ModuleDefinition {
    Name("NozyFfi")

    Function("nativeLibReady") { () -> Bool in
      // libnozy_ffi.a is not linked on iOS yet (needs macOS + cargo-lipo).
      false
    }

    Function("walletPaths") { () -> [String: String] in
      let dir = FileManager.default.urls(for: .documentDirectory, in: .userDomainMask)[0]
        .appendingPathComponent("nozy")
      try? FileManager.default.createDirectory(at: dir, withIntermediateDirectories: true)
      return [
        "walletDataDir": dir.path,
        "compactDbPath": dir.appendingPathComponent("lwd_compact.sqlite").path
      ]
    }

    Function("lockWallet") {}
  }
}
