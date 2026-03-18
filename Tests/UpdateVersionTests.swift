import XCTest
import Version

final class UpdateVersionTests: XCTestCase {

    // MARK: - Stable vs Stable

    func testNewerStableAvailable() {
        let current = Version(tolerant: "0.1.0")!
        let release = Version(tolerant: "0.2.0")!
        XCTAssertTrue(release > current)
    }

    func testSameVersionNoUpdate() {
        let current = Version(tolerant: "0.2.0")!
        let release = Version(tolerant: "0.2.0")!
        XCTAssertFalse(release > current)
    }

    func testCurrentNewerThanRelease() {
        let current = Version(tolerant: "0.3.0")!
        let release = Version(tolerant: "0.2.0")!
        XCTAssertFalse(release > current)
    }

    // MARK: - Pre-release ordering

    func testBetaIsLessThanStable() {
        let beta = Version(tolerant: "0.2.0-beta.3")!
        let stable = Version(tolerant: "0.2.0")!
        XCTAssertTrue(stable > beta, "0.2.0 should be newer than 0.2.0-beta.3")
    }

    func testBetaOrdering() {
        let beta3 = Version(tolerant: "0.2.0-beta.3")!
        let beta10 = Version(tolerant: "0.2.0-beta.10")!
        XCTAssertTrue(beta10 > beta3, "beta.10 should be newer than beta.3")
    }

    func testBetaOlderThanNextMinor() {
        let beta = Version(tolerant: "0.2.0-beta.5")!
        let next = Version(tolerant: "0.3.0")!
        XCTAssertTrue(next > beta)
    }

    // MARK: - Tolerant parsing

    func testStripVPrefix() {
        let tag = "v0.2.0"
        let stripped = tag.hasPrefix("v") ? String(tag.dropFirst()) : tag
        let version = Version(tolerant: stripped)
        XCTAssertNotNil(version)
        XCTAssertEqual(version, Version(0, 2, 0))
    }

    func testBetaTagParsing() {
        let tag = "v0.2.0-beta.3"
        let stripped = tag.hasPrefix("v") ? String(tag.dropFirst()) : tag
        let version = Version(tolerant: stripped)
        XCTAssertNotNil(version)
        XCTAssertEqual(version?.major, 0)
        XCTAssertEqual(version?.minor, 2)
        XCTAssertEqual(version?.patch, 0)
        XCTAssertEqual(version?.prereleaseIdentifiers, ["beta", "3"])
    }

    func testInvalidVersionReturnsNil() {
        XCTAssertNil(Version(tolerant: ""))
        XCTAssertNil(Version(tolerant: "not-a-version"))
    }

    // MARK: - Real-world scenarios

    func testBetaUserSeesStableUpdate() {
        // User on beta, stable release available
        let current = Version(tolerant: "0.2.0-beta.5")!
        let release = Version(tolerant: "0.2.0")!
        XCTAssertTrue(release > current, "Beta user should see stable 0.2.0 as update")
    }

    func testStableUserDoesNotSeeBeta() {
        // User on stable, only beta available (would not appear via /releases/latest anyway,
        // but verify comparison is correct)
        let current = Version(tolerant: "0.2.0")!
        let beta = Version(tolerant: "0.3.0-beta.1")!
        XCTAssertTrue(beta > current, "0.3.0-beta.1 is technically newer than 0.2.0")
        // Note: in practice, /releases/latest skips pre-releases so stable users won't see betas
    }

    func testPatchUpdate() {
        let current = Version(tolerant: "0.2.0")!
        let patch = Version(tolerant: "0.2.1")!
        XCTAssertTrue(patch > current)
    }
}
