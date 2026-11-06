# SFTP Deploy

Status: ready-for-agent

## Problem Statement

Patch Builder can build an Update tree locally, but a publisher must use another tool to move that Update tree to the server consumed by Launchers. This separates Build from publication, requires the publisher to determine which Archives changed, and makes it easy to upload too much, omit a required Archive, expose a partial file, or publish the Manifest before its Archives are available.

The application already has the information needed for an incremental publication: the local Manifest describes every current Source file and its selected hashes, while a previously deployed remote Manifest describes the server's published state. Patch Builder does not currently connect over SSH, compare those Manifests, verify the server identity, transfer Archives, or report transfer progress and throughput.

## Solution

Add a Deploy action beside the appearance control in the title bar. The action opens a modal where the publisher configures an SFTP destination and chooses password or private-key authentication. The modal validates the connection, verifies the SSH host key, reads the remote Manifest, compares it with the local Manifest by path and selected hashes, and uploads only new, changed, or missing Archives. The local Manifest is published last.

The operation reports its stage, current Archive, byte progress, transfer rate, and final publication statistics. Non-sensitive connection details are remembered; passwords and private-key passphrases remain in memory only. The remote Update tree remains a collection of static files for Launchers to consume over HTTP or a CDN. SFTP is only the publisher-facing transport.

## User Stories

1. As a publisher, I want a Deploy action beside the appearance control, so that publication is available from the main Patch Builder workflow.
2. As a publisher, I want Deploy to open a modal, so that connection details and transfer state remain focused in one place.
3. As a publisher, I want to enter an SSH host, so that I can choose the server receiving the Update tree.
4. As a publisher, I want to enter an SSH port with `22` as the default, so that standard and non-standard SFTP services are supported.
5. As a publisher, I want to enter an SSH user, so that Patch Builder can authenticate with the account authorized to publish.
6. As a publisher, I want to enter a remote destination, so that I can select where the remote Update tree is stored.
7. As a publisher, I want to authenticate with a password, so that I can use SFTP hosts configured for password authentication.
8. As a publisher, I want to authenticate with a private key, so that I can use hosts where password authentication is disabled.
9. As a publisher, I want to provide an optional private-key passphrase, so that encrypted private keys are supported.
10. As a publisher, I want to select my private-key file with a native file picker, so that I do not need to type its path manually.
11. As a publisher, I want password and passphrase fields to be masked, so that credentials are not exposed on screen.
12. As a publisher, I want passwords and passphrases to remain unsaved, so that Patch Builder does not place reusable credentials in its settings file.
13. As a publisher, I want non-sensitive connection details remembered, so that repeated deployments require minimal setup.
14. As an existing publisher, I want settings created before Deploy to continue loading, so that this feature does not invalidate my Source, Update tree, hash, or appearance preferences.
15. As a publisher, I want Patch Builder to verify the server's SSH host key, so that credentials and files are not sent to an impersonated server.
16. As a publisher connecting to an unknown host, I want to review its key algorithm and SHA-256 fingerprint before trusting it, so that first use is an explicit security decision.
17. As a publisher connecting to a host whose key changed, I want Deploy blocked with the expected and received identities shown, so that a possible interception is not silently accepted.
18. As a publisher, I want trusted host identities remembered independently from credentials, so that normal repeat deployments remain secure and convenient.
19. As a publisher, I want invalid or incomplete connection fields identified before connection, so that routine input errors are quick to correct.
20. As a publisher, I want Deploy to run outside the UI thread, so that the application remains responsive while connecting and transferring.
21. As a publisher, I want Scan, Build, and Deploy prevented from mutating shared publication state concurrently, so that one operation cannot invalidate another.
22. As a publisher with a successful Scan that found pending work, I want Patch Builder to require a Build before Deploy, so that I do not accidentally publish a stale Update tree.
23. As a publisher without a current Scan, I want to deploy an already built and valid Update tree, so that publication does not depend on rehashing an unavailable Source.
24. As a publisher, I want the local Manifest validated before remote writes, so that an invalid publication cannot partially modify the server.
25. As a publisher, I want every Archive required for the deployment checked locally before transfer, so that missing local output fails before publication.
26. As a publisher, I want a missing remote Manifest treated as the first deployment, so that an empty destination receives the complete Update tree.
27. As a publisher, I want an invalid remote Manifest reported rather than treated as an empty destination, so that existing server corruption is not silently overwritten.
28. As a publisher, I want local and remote Manifest entries matched by their relative path, so that each Archive is compared with its corresponding published file.
29. As a publisher using BLAKE3, I want Archive reuse decided by the BLAKE3 hash, so that timestamps and filesystem metadata do not cause redundant transfers.
30. As a publisher using CRC32, I want Archive reuse decided by the CRC32 hash, so that the selected Manifest contract controls deployment.
31. As a publisher using both algorithms, I want both hashes to match before an Archive is reused, so that the complete selected contract is respected.
32. As a publisher changing the selected hash algorithm, I want all Archives uploaded when the old and new hashes are not comparable, so that the remote Update tree is consistent with the new Manifest.
33. As a publisher, I want an Archive uploaded when its path is new or its comparable hash changed, so that changed Source content reaches Launchers.
34. As a publisher, I want an Archive uploaded when the remote Manifest claims a matching hash but the remote Archive is missing, so that the published tree repairs missing content.
35. As a publisher, I want matching Archives left untouched, so that repeated Deploys avoid unnecessary transfer time and preserve remote cache entries.
36. As a publisher, I want nested remote directories created as needed, so that the Source layout is reproduced without manual server preparation.
37. As a publisher, I want each Archive uploaded under a temporary name before installation, so that interrupted transfers do not expose partial public Archives.
38. As a publisher, I want the Manifest uploaded only after every required Archive is installed, so that Launchers never read a new Manifest before its Archives are available.
39. As a publisher, I want the Manifest installed through a temporary file and server-side rename, so that Launchers never read a partially uploaded Manifest.
40. As a publisher, I want files present only in the remote Update tree left untouched, so that Deploy sends new and modified content without deleting remote data.
41. As a publisher removing a Source file, I want the new Manifest published without that entry, so that Launchers stop treating the old Archive as current even though the stale remote Archive remains stored.
42. As a publisher, I want a deployment with identical Manifests and present Archives to perform no remote writes, so that an up-to-date destination remains untouched.
43. As a publisher, I want a newer remote Manifest or a same-version divergent Manifest called out before replacement, so that an accidental rollback or branch overwrite requires explicit confirmation.
44. As a publisher, I want Deploy to detect when the remote Manifest changes during my operation, so that another publisher's work is not silently overwritten.
45. As a publisher, I want to see the current connection or transfer stage, so that I know whether Deploy is connecting, comparing, uploading, or publishing.
46. As a publisher, I want to see the current Archive, completed Archive count, bytes, percentage, and transfer rate, so that I can judge progress on a large Update tree.
47. As a publisher, I want to cancel an active Deploy, so that I can stop a mistaken or unexpectedly slow publication without publishing its Manifest.
48. As a publisher, I want cancellation to take effect at a safe transfer boundary and clean temporary files where possible, so that cancellation does not advertise incomplete output.
49. As a publisher, I want successful Deploy results to show uploaded and reused Archive counts, bytes transferred, duration, average rate, Manifest version, and destination, so that I can verify what was published.
50. As a publisher, I want connection, host-key, authentication, Manifest, directory, Archive, and publication failures identified by stage, so that I can correct the actual problem.
51. As a publisher, I want a failed Deploy to preserve the last remote Manifest and retain my non-sensitive form values, so that Launchers keep the last published view and I can retry efficiently.
52. As a publisher, I want credentials omitted from errors and diagnostics, so that troubleshooting does not disclose secrets.
53. As a publisher, I want finite connection and I/O timeouts, so that an unreachable or stalled server does not leave Deploy running indefinitely.
54. As a publisher, I want the Deploy modal readable and operable in Light and Dark, so that publication respects my selected appearance.
55. As a keyboard or assistive-technology user, I want the Deploy action, modal fields, authentication choice, progress, confirmation, cancellation, and result exposed semantically, so that publication does not require pointer-only interaction.
56. As a publisher, I want title-bar dragging, appearance switching, and native window controls to keep working around the new Deploy action, so that custom window chrome does not regress.
57. As a publisher, I do not want a post-Deploy Open action, so that the completed state remains focused on publication facts without depending on an external SFTP client or public URL.
58. As a publisher, I want the optimized Patch Builder executable to include the SFTP capability without a separate runtime installation, so that deployment remains part of the distributed desktop tool.

## Implementation Decisions

- The feature introduces **Deploy** as a publisher-facing operation that copies a completed local Update tree to a remote Update tree over SFTP. It does not change how Launchers consume the Manifest and Archives over HTTP or a CDN.
- The Deploy action is a labeled title-bar control positioned beside the appearance control. It consumes its own pointer and keyboard interaction so that it cannot start title-bar dragging or maximization.
- Deploy uses an application-modal overlay with explicit configuration, host-trust, connecting, comparing, transferring, publishing, success, and failure states. The underlying application cannot start another publishing operation while the modal owns active work.
- The connection profile contains host, port, user, remote destination, authentication method, and private-key path. Port defaults to `22`.
- Credentials are a separate ephemeral input. Passwords and private-key passphrases are never part of saved settings, trusted-host storage, reports, or error text. They are cleared when the modal closes or the operation completes.
- Authentication supports exactly password and private-key file with an optional passphrase. SSH agent, keyboard-interactive authentication, and client-certificate management are not introduced.
- The existing saved-settings document gains a defaulted Deploy profile. Missing Deploy data preserves compatibility with older settings. Persisting Deploy configuration must not change Source, Update tree, hash selection, or appearance semantics.
- Trusted SSH host keys are stored in an application-owned OpenSSH-compatible known-hosts file, separate from the settings document. Patch Builder does not modify the user's global SSH configuration.
- Host-key verification happens after the SSH handshake and before authentication. Unknown keys require explicit trust based on key algorithm and SHA-256 fingerprint. A mismatch is a hard stop until the publisher explicitly replaces the stored trust.
- The SSH/SFTP implementation uses the blocking `ssh2` client API on the application's existing background-work model. A new asynchronous runtime is not introduced.
- The primary behavioral interface is one top-level Deploy operation. It accepts the local Update tree, connection profile, ephemeral credential, cancellation state, and progress sink, and returns either a publication report or a stage-specific error. This is the principal test seam.
- The Deploy operation owns connection, host verification, authentication, remote Manifest acquisition, comparison, transfer, and final Manifest publication. UI callbacks do not implement publication decisions.
- Manifest decoding and validation are shared between local filesystem reads and remote bytes so that local and remote documents obey one contract. A remote Manifest is read with a finite size limit before JSON parsing.
- Manifest paths used for deployment must be normalized relative `/`-separated paths. Empty paths, absolute paths, backslashes, platform prefixes, `.` segments, and `..` segments are rejected before they can influence a local or remote filesystem path.
- Remote paths use POSIX semantics regardless of the publisher's operating system. The configured destination may be absolute or relative to the SFTP account's home, but Manifest paths cannot escape it.
- Deploy reads the completed local Manifest from the Update tree and verifies the corresponding local Archive for every entry that may be transferred. It never rebuilds Archives from the Source.
- A successful current Scan with a non-empty Plan blocks Deploy until Build publishes that Plan. If no successful current Scan exists, a valid existing Update tree may still be deployed.
- A remote `manifest.json` absence means no prior deployment. Other read failures and invalid JSON are errors, not first-deployment fallbacks.
- Entries are paired by exact Manifest path. Change detection uses only the hash fields selected by the local and remote Manifests; size, modification time, and remote file timestamp never establish content equality.
- BLAKE3 compares the primary hash, CRC32 compares the primary hash, and the combined algorithm requires both the primary BLAKE3 hash and CRC32 field to match.
- Manifests using different hash algorithms are not comparable. Every local Archive is selected for upload before the local Manifest replaces the remote one.
- A hash-matching entry is reused only when its remote Archive also exists as a regular file. The remote Archive is not downloaded and rehashed.
- Remote-only Manifest entries select no deletion. Publishing the local Manifest makes those entries non-current, but their old Archives remain on remote storage.
- The Deploy plan records new, changed, missing, and reused Archives, the real local Archive byte total, whether the Manifest differs, and the initial remote Manifest identity. Progress uses compressed Archive bytes actually transferred rather than uncompressed Source sizes from the Manifest.
- Directories are created recursively with conventional server permissions before their files are installed.
- Every selected Archive is streamed sequentially through one SFTP session to a unique temporary sibling, synchronized where supported, and renamed to its public path. The implementation does not load a complete Archive into memory.
- Replacement requires server-side rename semantics that do not delete the valid destination before the temporary upload is ready. If the server cannot safely replace a destination, Deploy reports incompatibility and leaves the old public file intact.
- The local Manifest is serialized from the validated local contract, uploaded to a unique temporary sibling, and renamed to `manifest.json` only after every selected Archive has been installed.
- Immediately before publishing the Manifest, Deploy rereads the remote Manifest and verifies that it still matches the document observed during planning. A change aborts publication as a concurrent-publisher conflict.
- A missing remote Manifest permits first deployment. An identical semantic Manifest with every Archive present produces an up-to-date report and no writes. A differing Manifest is published even when no Archive transfer is needed, such as when local removal leaves only a stale remote Archive.
- A remote version greater than the local version, or an equal version with different Manifest content, produces a conflict requiring explicit publisher confirmation before the top-level operation is retried with replacement authorized. Silence is never treated as authorization.
- Cancellation is cooperative. It is checked while streaming and between SFTP operations; the connection is closed, temporary files are removed in best effort, and the local Manifest is not published. A blocking operation may take until its configured timeout to return.
- Progress is monotonic and reports stage, current Archive, completed and total Archive counts, transferred and total bytes, fraction complete, and elapsed-time throughput. Credentials and private-key contents never enter progress data.
- The final report includes previous and published Manifest versions, new and changed Archive counts, reused count, transferred bytes, elapsed duration, average transfer rate, and normalized destination display.
- Errors identify the failing stage and relevant non-secret host or path. Partial Archive installation may have occurred, but failure before the final rename preserves the previously visible remote Manifest.
- Scan, Build, and Deploy are mutually exclusive. Appearance switching remains presentation-only and may remain available while Deploy runs if it cannot dismiss or mutate the modal.
- The modal uses the existing semantic theme values and adds semantic values only where its states require them. It must remain readable in Light and Dark and expose accessible labels, descriptions, focus order, default action, cancellation, status, and progress.
- The completed state has Close and retry/new-Deploy behavior as appropriate, but no Open action.
- Product documentation gains the Deploy workflow and security behavior. Domain documentation defines Deploy and remote Update tree. An architectural decision records hash-based SFTP comparison, temporary installation, Manifest-last publication, and the deliberate no-deletion policy.
- Release validation must prove that the optimized distributed executable contains its required SSH implementation and does not depend on an unshipped application-specific runtime component.

## Testing Decisions

- Tests assert publisher-visible publication behavior, security boundaries, compatibility, and remote filesystem outcomes. They do not assert source text, callback forwarding, internal field names, exact temporary filenames, private helper calls, or Slint binding structure.
- The highest and primary automated seam is the top-level Deploy operation exercised against a disposable real SFTP server. This keeps connection, authentication, Manifest comparison, directory creation, streaming, rename, publication order, reporting, and error behavior in one observable scenario rather than mocking transport calls.
- The disposable SFTP environment supports one password-authenticated account and one private-key-authenticated account with an encrypted-key case. Test data uses a real locally built Update tree and inspects the resulting remote Manifest and Archive bytes.
- A first-deployment test proves that all Archives arrive, nested directories are created, and the Manifest is readable only after the operation completes.
- An unchanged repeat test proves that matching Archives are reused and no remote file is rewritten.
- Incremental tests cover one new Archive, one changed Archive, one missing remote Archive, and a local removal that publishes only the changed Manifest while retaining the stale remote Archive.
- Hash-selection tests cover BLAKE3, CRC32, both algorithms, and an algorithm transition that selects every local Archive.
- Equality tests prove that timestamps and sizes are not used as substitutes for comparable hashes. Existence remains a separate repair condition.
- Failure tests interrupt a transfer and reject final rename, then verify that no partial file has the public temporary target name and that the previous remote Manifest remains visible.
- Concurrent-change coverage modifies the remote Manifest after planning and verifies that final publication is refused.
- Conflict coverage verifies that newer remote versions and same-version divergent content require explicit replacement authorization, while ordinary older versions proceed.
- Security tests cover a first-use host fingerprint, a repeated matching host key, a changed host key, wrong password, wrong key, wrong passphrase, and omission of credentials from errors and reports.
- Validation tests cover malformed local and remote Manifests, over-limit remote Manifest data, missing local Archives, absolute paths, parent traversal, platform prefixes, and destination containment.
- Cancellation coverage stops a multi-Archive deployment after progress begins, verifies the final Manifest was not published, and accepts best-effort cleanup only for temporary artifacts.
- Timeout coverage proves that unreachable and stalled endpoints terminate with stage-specific errors rather than leaving the operation indefinitely active.
- Saved-settings serialization remains a separate existing compatibility seam because it is the public persistence contract. A legacy document defaults to an empty Deploy profile and port `22`; each non-secret authentication profile round-trips; serialized JSON never contains password or passphrase fields.
- The running desktop application is the UI acceptance seam. A smoke scenario exercises opening and closing the modal, password and private-key forms, native key selection, unknown-host confirmation, conflict confirmation, live progress, cancellation, failure, retry, success statistics, keyboard navigation, accessible labels, and Light/Dark rendering.
- The UI smoke also verifies that Deploy does not trigger title-bar dragging, that window controls and appearance switching retain their behavior, and that Scan and Build cannot start during active Deploy work.
- A workflow smoke verifies that a pending Plan requires Build first, while a valid previously built Update tree can be deployed when no successful current Scan exists.
- The existing saved-settings tests are prior art for compatibility and round-trip assertions. Existing Scan and Build tests are prior art for temporary publication, progress, Manifest-last invariants, and observable filesystem results.
- Slint compilation remains the contract check for component properties, callbacks, resources, and accessible bindings. Screenshot-source tests and tests that merely assert copied values are not added.
- The full Rust test suite remains the regression check for Source scanning, Plan construction, Build, Archive reuse, Manifest serialization, settings, and formatting.
- The optimized release build and the produced executable are exercised against the disposable SFTP server to verify native dependency packaging and actual deployment from the shipped artifact.

## Out of Scope

- Deleting remote Archives or directories that are absent from the local Manifest.
- Reclaiming storage occupied by stale remote Archives.
- Downloading and rehashing remote Archives to detect corruption when the remote Manifest and file existence claim they are current.
- Browsing the remote filesystem or editing remote files inside Patch Builder.
- A post-Deploy Open button, public-URL field, browser launch, or external SFTP-client launch.
- Running a web server, SSH server, HTTP upload endpoint, or CDN inside Patch Builder.
- CDN invalidation, cache purge, or provider-specific deployment APIs.
- Scheduled, automatic, command-line, or unattended deployment.
- Multiple named deployment environments or connection profiles.
- Persisting passwords, private-key passphrases, private-key contents, or tokens.
- SSH agent, keyboard-interactive, hardware-key, certificate, jump-host, proxy, or multi-factor authentication.
- Parallel multi-session uploads, bandwidth limiting, resumable uploads, or delta transfer within an Archive.
- Distributed locking across unrelated publisher tools. Deploy detects a changed Manifest before final publication but does not control external writers.
- Rolling back to an earlier remote Update tree or retaining release history.
- Changing Manifest schema, Archive naming, compression, selected hash semantics, or Launcher behavior.
- Rebuilding Source files as part of Deploy. Build remains the only operation that changes the local Update tree.

## Further Notes

- “Deploy” and “remote Update tree” are new domain terms. Deploy publishes the completed local Update tree; it is not a synonym for Build. The remote Update tree has the same Manifest-plus-Archives shape as the local Update tree.
- The remote Manifest is an index, not proof of remote Archive bytes. A matching Manifest hash plus an existing Archive is trusted to avoid downloading the remote content. Repairing undetected remote corruption remains outside this feature.
- Manifest-last publication preserves the repository's existing direction of safety: a newly visible Manifest never intentionally references an Archive that was not installed first. Because public Archive paths are stable, replacing a changed Archive before the Manifest still cannot provide a transaction across the entire tree; fully immutable hash-addressed Archive paths would require a separate Launcher contract change.
- Remote files excluded by the local Manifest become unreachable through the current Launcher contract but continue consuming server storage. This is deliberate because the requested Deploy sends only new and modified content.
- Common OpenSSH SFTP servers support the required temporary-file and server-side rename flow. Servers without safe replacement semantics fail explicitly rather than falling back to direct writes of public files.
- The implementation should use finite TCP, SSH, and SFTP timeouts and avoid exposing low-level library codes without the operation stage and actionable context.
- Documentation must make clear that trusting a first-use fingerprint is a security decision and that a changed fingerprint should be verified out of band before replacement.
