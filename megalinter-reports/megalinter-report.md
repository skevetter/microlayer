## ✅⚠️[MegaLinter](https://megalinter.io/9.0.1) analysis: Success with warnings



|  Descriptor   |                                               Linter                                                |Files|Fixed|Errors|Warnings|Elapsed time|
|---------------|-----------------------------------------------------------------------------------------------------|----:|----:|-----:|-------:|-----------:|
|✅ ACTION      |[actionlint](https://megalinter.io/9.0.1/descriptors/action_actionlint)                              |    4|     |     0|       0|       0.36s|
|⚠️ COPYPASTE   |[jscpd](https://megalinter.io/9.0.1/descriptors/copypaste_jscpd)                                     |  yes|     |    18|      no|       2.51s|
|⚠️ DOCKERFILE  |[hadolint](https://megalinter.io/9.0.1/descriptors/dockerfile_hadolint)                              |    1|     |     1|       0|       0.36s|
|⚠️ EDITORCONFIG|[editorconfig-checker](https://megalinter.io/9.0.1/descriptors/editorconfig_editorconfig_checker)    |   27|     |     1|       0|       0.41s|
|✅ MARKDOWN    |[markdownlint](https://megalinter.io/9.0.1/descriptors/markdown_markdownlint)                        |    1|    1|     0|       0|       0.58s|
|✅ MARKDOWN    |[markdown-table-formatter](https://megalinter.io/9.0.1/descriptors/markdown_markdown_table_formatter)|    1|    1|     0|       0|       0.24s|
|✅ REPOSITORY  |[devskim](https://megalinter.io/9.0.1/descriptors/repository_devskim)                                |  yes|     |    no|      no|       1.41s|
|✅ REPOSITORY  |[dustilock](https://megalinter.io/9.0.1/descriptors/repository_dustilock)                            |  yes|     |    no|      no|       0.25s|
|✅ REPOSITORY  |[gitleaks](https://megalinter.io/9.0.1/descriptors/repository_gitleaks)                              |  yes|     |    no|      no|       0.24s|
|✅ REPOSITORY  |[git_diff](https://megalinter.io/9.0.1/descriptors/repository_git_diff)                              |  yes|     |    no|      no|       0.02s|
|⚠️ REPOSITORY  |[grype](https://megalinter.io/9.0.1/descriptors/repository_grype)                                    |  yes|     |     1|      no|      37.16s|
|⚠️ REPOSITORY  |[kics](https://megalinter.io/9.0.1/descriptors/repository_kics)                                      |  yes|     |    16|      no|       4.37s|
|✅ REPOSITORY  |[secretlint](https://megalinter.io/9.0.1/descriptors/repository_secretlint)                          |  yes|     |    no|      no|       1.37s|
|✅ REPOSITORY  |[syft](https://megalinter.io/9.0.1/descriptors/repository_syft)                                      |  yes|     |    no|      no|       3.78s|
|⚠️ REPOSITORY  |[trivy](https://megalinter.io/9.0.1/descriptors/repository_trivy)                                    |  yes|     |     1|      no|       9.34s|
|✅ REPOSITORY  |[trivy-sbom](https://megalinter.io/9.0.1/descriptors/repository_trivy_sbom)                          |  yes|     |    no|      no|       6.49s|
|✅ REPOSITORY  |[trufflehog](https://megalinter.io/9.0.1/descriptors/repository_trufflehog)                          |  yes|     |    no|      no|      16.65s|
|✅ RUST        |[clippy](https://megalinter.io/9.0.1/descriptors/rust_clippy)                                        |  yes|     |    no|      no|     127.45s|
|⚠️ SPELL       |[cspell](https://megalinter.io/9.0.1/descriptors/spell_cspell)                                       |   28|     |   182|       0|       6.64s|
|✅ SPELL       |[lychee](https://megalinter.io/9.0.1/descriptors/spell_lychee)                                       |    6|     |     0|       0|       1.78s|
|✅ YAML        |[prettier](https://megalinter.io/9.0.1/descriptors/yaml_prettier)                                    |    5|    2|     0|       0|       0.57s|
|✅ YAML        |[v8r](https://megalinter.io/9.0.1/descriptors/yaml_v8r)                                              |    5|     |     0|       0|        4.5s|
|✅ YAML        |[yamllint](https://megalinter.io/9.0.1/descriptors/yaml_yamllint)                                    |    5|     |     0|       0|       0.54s|

## Detailed Issues

<details>
<summary>⚠️ SPELL / cspell - 182 errors</summary>

```
.github/workflows/ci.yml:22:32     - Unknown word (clippy)     -- components: rustfmt, clippy
	 Suggestions: [chippy, lippy, flippy, slippy, clip]
.github/workflows/ci.yml:27:15     - Unknown word (Clippy)     -- - name: Clippy
	 Suggestions: [Chippy, Lippy, Flippy, Slippy, Clip]
.github/workflows/ci.yml:28:20     - Unknown word (clippy)     -- run: cargo clippy -- -D warnings
	 Suggestions: [chippy, lippy, flippy, slippy, clip]
.github/workflows/ci.yml:38:44     - Unknown word (picolayer)  -- c%s "target/release/picolayer")
	 Suggestions: [player, picoline, pillager]
.github/workflows/docker.yml:24:29     - Unknown word (Buildx)     -- name: Set up Docker Buildx
	 Suggestions: [Build, Builds, Built, Build's, Builded]
.github/workflows/docker.yml:25:28     - Unknown word (buildx)     -- uses: docker/setup-buildx-action@v3
	 Suggestions: [build, builds, built, build's, builded]
.github/workflows/docker.yml:35:15     - Unknown word (dtolnay)    -- uses: dtolnay/rust-toolchain@stable
	 Suggestions: [tolan, tonya, dona, dtoa, delay]
.github/workflows/docker.yml:39:21     - Unknown word (picolayer)  -- - name: Build picolayer binary
	 Suggestions: [player, picoline, pillager]
.github/workflows/docker.yml:43:57     - Unknown word (picolayer)  -- unknown-linux-gnu/release/picolayer .
	 Suggestions: [player, picoline, pillager]
.github/workflows/release-nightly.yml:35:15     - Unknown word (dtolnay)    -- - uses: dtolnay/rust-toolchain@stable
	 Suggestions: [tolan, tonya, dona, dtoa, delay]
.github/workflows/release-nightly.yml:70:66     - Unknown word (picolayer)  -- matrix.target }}/release/picolayer" 2>/dev/null || stat
	 Suggestions: [player, picoline, pillager]
.github/workflows/release-nightly.yml:70:139    - Unknown word (picolayer)  -- matrix.target }}/release/picolayer")"
	 Suggestions: [player, picoline, pillager]
.github/workflows/release-nightly.yml:80:19     - Unknown word (picolayer)  -- tar czf picolayer-${{ matrix.target }
	 Suggestions: [player, picoline, pillager]
.github/workflows/release-nightly.yml:80:57     - Unknown word (picolayer)  -- matrix.target }}.tar.gz picolayer
	 Suggestions: [player, picoline, pillager]
.github/workflows/release-nightly.yml:81:14     - Unknown word (picolayer)  -- mv picolayer-${{ matrix.target }
	 Suggestions: [player, picoline, pillager]
.github/workflows/release-nightly.yml:119:15    - Unknown word (softprops)  -- uses: softprops/action-gh-release@v
	 Suggestions: [softphone, softwoods, southrons, softphones]
.github/workflows/release.yml:63:19     - Unknown word (picolayer)  -- tar czf picolayer-${{ matrix.target }
	 Suggestions: [player, picoline, pillager]
.github/workflows/release.yml:63:57     - Unknown word (picolayer)  -- matrix.target }}.tar.gz picolayer
	 Suggestions: [player, picoline, pillager]
.github/workflows/release.yml:64:14     - Unknown word (picolayer)  -- mv picolayer-${{ matrix.target }
	 Suggestions: [player, picoline, pillager]
.github/workflows/release.yml:69:17     - Unknown word (picolayer)  -- name: picolayer-${{ matrix.target }
	 Suggestions: [player, picoline, pillager]
.github/workflows/release.yml:70:17     - Unknown word (picolayer)  -- path: picolayer-${{ matrix.target }
	 Suggestions: [player, picoline, pillager]
.github/workflows/release.yml:94:13     - Unknown word (elif)       -- elif [ "$RELEASE_TYPE" =
	 Suggestions: [leif, elia, elis, eliz, enif]
.github/workflows/release.yml:119:15    - Unknown word (softprops)  -- uses: softprops/action-gh-release@v
	 Suggestions: [softphone, softwoods, southrons, softphones]
Cargo.toml:2:9       - Unknown word (picolayer)  -- name = "picolayer"
	 Suggestions: [player, picoline, pillager]
Cargo.toml:5:13      - Unknown word (skevetter)  -- authors = ["skevetter"]
	 Suggestions: [skeeter, sheeter, skelter, sweeter, skeeters]
Cargo.toml:18:1      - Unknown word (flate)      -- flate2 = "1.1.4"
	 Suggestions: [flat*, fate, flake, flame, flare]
Cargo.toml:20:1      - Unknown word (indicatif)  -- indicatif = { version = "0.17
	 Suggestions: [indicate, indicated, indicates, indicator, indicating]
Cargo.toml:21:1      - Unknown word (libpkgx)    -- libpkgx = { version = "0.7.
	 Suggestions: [libpng, Libpng, limpkin]
Cargo.toml:24:1      - Unknown word (reqwest)    -- reqwest = { version = "0.12
	 Suggestions: [request, reest, rawest, retest, revest]
Cargo.toml:24:67     - Unknown word (rustls)     -- blocking", "json", "rustls-tls-webpki-roots"],
	 Suggestions: [rusts, rust's, rustle, rustles, russ]
Cargo.toml:24:78     - Unknown word (webpki)     -- "json", "rustls-tls-webpki-roots"], default-features
	 Suggestions: [webkit, weak, webb, webm, webs]
Cargo.toml:25:1      - Unknown word (rusqlite)   -- rusqlite = { version = "0.33
	 Suggestions: [ruralite, sqlite, SQLite, uralite, ursuline]
Cargo.toml:26:1      - Unknown word (serde)      -- serde = { version = "1.0.
	 Suggestions: [sered, sere, serge, serve, verde]
Cargo.toml:27:1      - Unknown word (serde)      -- serde_json = "1.0.145"
	 Suggestions: [sered, sere, serge, serve, verde]
Cargo.toml:32:1      - Unknown word (walkdir)    -- walkdir = "2.5.0"
	 Suggestions: [walker, workdir, Walker, workDir, walkyrie]
dc09182e-3403-433c-9870-5d2c2f72dac6-megalinter_file_names_cspell.txt:12:23     - Unknown word (pkgx)       -- assets test artifacts pkgx 2 7 0 linux x86 64 tar
	 Suggestions: [pkg, paxg, page, payx, pegs]
dc09182e-3403-433c-9870-5d2c2f72dac6-megalinter_file_names_cspell.txt:13:23     - Unknown word (pkgx)       -- assets test artifacts pkgx 2 7 0 linux x86 64 tar
	 Suggestions: [pkg, paxg, page, payx, pegs]
Dockerfile:10:6      - Unknown word (picolayer)  -- COPY picolayer /usr/local/bin/picolayer
	 Suggestions: [player, picoline, pillager]
Dockerfile:10:31     - Unknown word (picolayer)  -- picolayer /usr/local/bin/picolayer
	 Suggestions: [player, picoline, pillager]
Dockerfile:12:29     - Unknown word (picolayer)  -- chmod +x /usr/local/bin/picolayer
	 Suggestions: [player, picoline, pillager]
Dockerfile:14:29     - Unknown word (picolayer)  -- ENTRYPOINT ["/usr/local/bin/picolayer"]
	 Suggestions: [player, picoline, pillager]
LICENSE:3:20      - Unknown word (skevetter)  -- Copyright (c) 2025 skevetter
	 Suggestions: [skeeter, sheeter, skelter, sweeter, skeeters]
README.md:1:3       - Unknown word (picolayer)  -- # picolayer
	 Suggestions: [player, picoline, pillager]
README.md:3:44      - Unknown word (Picolayer)  -- layer management tool. Picolayer helps keep container
	 Suggestions: [Player, Picoline, Pillager]
README.md:5:34      - Unknown word (nanolayer)  -- is inspired by the [nanolayer](https://github.com
	 Suggestions: [annoyer, nanometer]
README.md:13:34     - Unknown word (pkgx)       -- Execute commands with pkgx for automatic dependency
	 Suggestions: [pkg, paxg, page, payx, pegs]
README.md:35:1      - Unknown word (picolayer)  -- picolayer apt-get htop,curl,git
	 Suggestions: [player, picoline, pillager]
README.md:41:1      - Unknown word (picolayer)  -- picolayer apt-get neovim --ppas
	 Suggestions: [player, picoline, pillager]
README.md:41:28     - Unknown word (ppas)       -- picolayer apt-get neovim --ppas ppa:neovim-ppa/stable
	 Suggestions: [paps, pas, peas, ppam, pyas]
README.md:47:1      - Unknown word (picolayer)  -- picolayer apt-get neovim --ppas
	 Suggestions: [player, picoline, pillager]
README.md:47:28     - Unknown word (ppas)       -- picolayer apt-get neovim --ppas ppa:neovim-ppa/stable
	 Suggestions: [paps, pas, peas, ppam, pyas]
README.md:47:63     - Unknown word (ppas)       -- neovim-ppa/stable --force-ppas-on-non-ubuntu
	 Suggestions: [paps, pas, peas, ppam, pyas]
README.md:55:1      - Unknown word (picolayer)  -- picolayer apk htop,curl,git
	 Suggestions: [player, picoline, pillager]
README.md:81:22     - Unknown word (jesseduffield) -- picolayer gh-release jesseduffield/lazygit lazygit --version
	 Suggestions: []
README.md:81:36     - Unknown word (lazygit)       -- release jesseduffield/lazygit lazygit --version latest
	 Suggestions: [lait, lazy, lazio, layout, lazuli]
README.md:81:44     - Unknown word (lazygit)       -- jesseduffield/lazygit lazygit --version latest --checksum
	 Suggestions: [lait, lazy, lazio, layout, lazuli]
README.md:99:22     - Unknown word (pkgxdev)       -- picolayer gh-release pkgxdev/pkgx pkgx --version
	 Suggestions: [pkgadd, pkg-dir]
README.md:99:30     - Unknown word (pkgx)          -- picolayer gh-release pkgxdev/pkgx pkgx --version latest
	 Suggestions: [pkg, paxg, page, payx, pegs]
README.md:99:35     - Unknown word (pkgx)          -- release pkgxdev/pkgx pkgx --version latest --checksum
	 Suggestions: [pkg, paxg, page, payx, pegs]
README.md:102:23    - Unknown word (pkgx)          -- # Run commands with pkgx
	 Suggestions: [pkg, paxg, page, payx, pegs]
README.md:104:35    - Unknown word (pkgx)          -- version of any tool using pkgx for automatic dependency
	 Suggestions: [pkg, paxg, page, payx, pegs]
README.md:144:24    - Unknown word (pyproject)     -- requirements.txt`, `pyproject.toml` → Python
	 Suggestions: [project, projects]
src/gh_release.rs:2:5       - Unknown word (flate)      -- use flate2::read::GzDecoder;
	 Suggestions: [flat*, fate, flake, flame, flare]
src/gh_release.rs:4:5       - Unknown word (reqwest)    -- use reqwest::blocking::Client;
	 Suggestions: [request, reest, rawest, retest, revest]
src/gh_release.rs:140:36    - Unknown word (picolayer)  -- header("User-Agent", "picolayer")
	 Suggestions: [player, picoline, pillager]
src/gh_release.rs:265:35    - Unknown word (armv)       -- arm" => vec!["arm", "armv7"],
	 Suggestions: [argv, arms, army, arm, Arm]
src/gh_release.rs:309:36    - Unknown word (picolayer)  -- header("User-Agent", "picolayer")
	 Suggestions: [player, picoline, pillager]
src/gh_release.rs:407:22    - Unknown word (walkdir)    -- for entry in walkdir::WalkDir::new(extract
	 Suggestions: [walker, workdir, Walker, workDir, walkyrie]
src/gh_release.rs:572:36    - Unknown word (picolayer)  -- header("User-Agent", "picolayer")
	 Suggestions: [player, picoline, pillager]
src/gh_release.rs:587:36    - Unknown word (picolayer)  -- header("User-Agent", "picolayer")
	 Suggestions: [player, picoline, pillager]
src/gh_release.rs:666:36    - Unknown word (picolayer)  -- header("User-Agent", "picolayer")
	 Suggestions: [player, picoline, pillager]
src/gh_release.rs:721:15    - Unknown word (lzma)       -- ".tar.lzma",
	 Suggestions: [lama, lima, Lima, liam, loam]
src/gh_release.rs:728:11    - Unknown word (lzma)       -- ".lzma",
	 Suggestions: [lama, lima, Lima, liam, loam]
src/main.rs:12:19     - Unknown word (picolayer)  -- #[command(name = "picolayer")]
	 Suggestions: [player, picoline, pillager]
src/main.rs:30:9      - Unknown word (ppas)       -- ppas: Option<String>,
	 Suggestions: [paps, pas, peas, ppam, pyas]
src/main.rs:34:15     - Unknown word (ppas)       -- force_ppas_on_non_ubuntu: bool
	 Suggestions: [paps, pas, peas, ppam, pyas]
src/main.rs:79:29     - Unknown word (pkgx)       -- Run a command using pkgx
	 Suggestions: [pkgs, pkg, paxg, page, payx]
src/main.rs:96:19     - Unknown word (pkgx)       -- /// Force pkgx even if dependencies
	 Suggestions: [pkgs, pkg, paxg, page, payx]
src/main.rs:98:15     - Unknown word (pkgx)       -- force_pkgx: bool,
	 Suggestions: [pkgs, pkg, paxg, page, payx]
src/main.rs:104:34    - Unknown word (pkgx)       -- Completely uninstall pkgx and remove all cache
	 Suggestions: [pkgs, pkg, paxg, page, payx]
src/main.rs:116:13    - Unknown word (ppas)       -- ppas,
	 Suggestions: [paps, pas, peas, ppam, pyas]
src/main.rs:117:19    - Unknown word (ppas)       -- force_ppas_on_non_ubuntu,
	 Suggestions: [paps, pas, peas, ppam, pyas]
src/main.rs:122:17    - Unknown word (ppas)       -- ppas.map(|p| p.split(','
	 Suggestions: [paps, pas, peas, ppam, pyas]
src/main.rs:169:19    - Unknown word (pkgx)       -- force_pkgx,
	 Suggestions: [pkgs, pkg, paxg, page, payx]
src/run.rs:8:18      - Unknown word (pkgx)       -- pub fn uninstall_pkgx() -> Result<()> {
	 Suggestions: [pkgs, pkg, paxg, page, payx]
src/run.rs:9:28      - Unknown word (pkgx)       -- println!("Uninstalling pkgx and removing all associated
	 Suggestions: [pkgs, pkg, paxg, page, payx]
src/run.rs:14:42     - Unknown word (pkgx)       -- vec!["/usr/local/bin/pkgx", "/usr/local/bin/pkgm
	 Suggestions: [pkgs, pkg, paxg, page, payx]
src/run.rs:14:65     - Unknown word (pkgm)       -- pkgx", "/usr/local/bin/pkgm"];
	 Suggestions: [pkgs, pkg, kpmg, page, palm]
src/run.rs:33:13     - Unknown word (pkgx)       -- let pkgx_dir = home_dir.join
	 Suggestions: [pkgs, pkg, paxg, page, payx]
src/run.rs:33:40     - Unknown word (pkgx)       -- dir = home_dir.join(".pkgx");
	 Suggestions: [pkgs, pkg, paxg, page, payx]
src/run.rs:186:17    - Unknown word (gofmt)      -- "go" | "gofmt" => "go.dev".to_string
	 Suggestions: [gift, goat, goff, gout, govt]
src/run.rs:187:19    - Unknown word (javac)      -- "java" | "javac" | "mvn" | "gradle"
	 Suggestions: [java, javan, javas, Java, Javan]
src/run.rs:188:20    - Unknown word (rustc)      -- "cargo" | "rustc" => "rust-lang.org"
	 Suggestions: [rust, rusts, rusty, rustic, Rusty]
src/run.rs:208:15    - Unknown word (libpkgx)    -- match try_libpkgx_execution(
	 Suggestions: [libpng, Libpng, limpkin]
src/run.rs:228:8     - Unknown word (libpkgx)    -- fn try_libpkgx_execution(
	 Suggestions: [libpng, Libpng, limpkin]
src/run.rs:255:82    - Unknown word (shellenv)   -- set by Mise or other shellenv tools
	 Suggestions: [shelled, sheller, shelley, suellen, shellers]
src/run.rs:256:42    - Unknown word (GOROOT)     -- == "go" && (key == "GOROOT" || key == "GOPATH"
	 Suggestions: [GROT, GODOT, Godot, COROT, Corot]
src/run.rs:256:61    - Unknown word (GOPATH)     -- "GOROOT" || key == "GOPATH") {
	 Suggestions: [GATH, GOAT, GOTH, GOATS, GOPAK]
src/run.rs:268:55    - Unknown word (libpkgx)    -- resolve_package_with_libpkgx(&[tool_spec]) {
	 Suggestions: [libpng, Libpng, limpkin]
src/run.rs:274:26    - Unknown word (GOROOT)     -- // Overwrite GOROOT for Go installations
	 Suggestions: [GROT, GODOT, Godot, COROT, Corot]
src/run.rs:274:90    - Unknown word (shellenv)   -- conflicts with Mise or other shellenv tools
	 Suggestions: [shelled, sheller, shelley, suellen, shellers]
src/run.rs:279:30    - Unknown word (GOROOT)     -- "GOROOT".to_string(),
	 Suggestions: [GROT, GODOT, Godot, COROT, Corot]
src/run.rs:306:58    - Unknown word (libpkgx)    -- resolved package with libpkgx");
	 Suggestions: [libpng, Libpng, limpkin]
src/run.rs:324:58    - Unknown word (libpkgx)    -- execute command with libpkgx")?;
	 Suggestions: [libpng, Libpng, limpkin]
src/run.rs:389:14    - Unknown word (indicatif)  -- bar: indicatif::ProgressBar,
	 Suggestions: [indicate, indicated, indicates, indicator, indicating]
src/run.rs:394:23    - Unknown word (indicatif)  -- let bar = indicatif::ProgressBar::new(0
	 Suggestions: [indicate, indicated, indicates, indicator, indicating]
src/run.rs:396:17    - Unknown word (indicatif)  -- indicatif::ProgressStyle::with
	 Suggestions: [indicate, indicated, indicates, indicator, indicating]
src/run.rs:418:20    - Unknown word (rusqlite)   -- let mut conn = rusqlite::Connection::open(&config
	 Suggestions: [ruralite, sqlite, SQLite, uralite, ursuline]
src/run.rs:428:21    - Unknown word (reqs)       -- let mut package_reqs = Vec::new();
	 Suggestions: [re's, rebs, recs, reds, refs]
src/run.rs:431:32    - Unknown word (reqs)       -- Ok(req) => package_reqs.push(req),
	 Suggestions: [re's, rebs, recs, reds, refs]
src/run.rs:439:16    - Unknown word (reqs)       -- if package_reqs.is_empty() {
	 Suggestions: [re's, rebs, recs, reds, refs]
src/run.rs:443:55    - Unknown word (reqs)       -- hydrate::hydrate(&package_reqs, |project| {
	 Suggestions: [re's, rebs, recs, reds, refs]
src/run.rs:621:41    - Unknown word (rustc)      -- map_tool_to_project("rustc"), "rust-lang.org")
	 Suggestions: [rust, rusts, rusty, rustic, Rusty]
src/utils/linux_info.rs:5:15      - Unknown word (Distro)     -- pub enum LinuxDistro {
	 Suggestions: [Distr, Distort, Dist, Disco, Ditto]
src/utils/linux_info.rs:13:15     - Unknown word (distro)     -- pub fn detect_distro() -> Result<LinuxDistro
	 Suggestions: [distr, distort, dist, disco, ditto]
src/utils/linux_info.rs:13:39     - Unknown word (Distro)     -- distro() -> Result<LinuxDistro> {
	 Suggestions: [Distr, Distort, Dist, Disco, Ditto]
src/utils/linux_info.rs:16:28     - Unknown word (Distro)     -- return Ok(LinuxDistro::Ubuntu);
	 Suggestions: [Distr, Distort, Dist, Disco, Ditto]
src/utils/linux_info.rs:18:28     - Unknown word (Distro)     -- return Ok(LinuxDistro::Alpine);
	 Suggestions: [Distr, Distort, Dist, Disco, Ditto]
src/utils/linux_info.rs:24:28     - Unknown word (Distro)     -- return Ok(LinuxDistro::Debian);
	 Suggestions: [Distr, Distort, Dist, Disco, Ditto]
src/utils/linux_info.rs:33:21     - Unknown word (distro)     -- matches!(detect_distro(), Ok(LinuxDistro::Ubuntu
	 Suggestions: [distr, distort, dist, disco, ditto]
src/utils/linux_info.rs:39:16     - Unknown word (distro)     -- detect_distro(),
	 Suggestions: [distr, distort, dist, disco, ditto]
src/utils/linux_info.rs:46:21     - Unknown word (distro)     -- matches!(detect_distro(), Ok(LinuxDistro::Alpine
	 Suggestions: [distr, distort, dist, disco, ditto]
src/utils/linux_info.rs:54:19     - Unknown word (distro)     -- fn test_linux_distro_enum() {
	 Suggestions: [distr, distort, dist, disco, ditto]
tests/apk_test.rs:4:17      - Unknown word (picolayer)  -- use common::run_picolayer;
	 Suggestions: [player, picoline, pillager]
tests/apk_test.rs:22:22     - Unknown word (picolayer)  -- let output = run_picolayer(&["apk", "curl"]);
	 Suggestions: [player, picoline, pillager]
tests/apk_test.rs:53:22     - Unknown word (picolayer)  -- let output = run_picolayer(&["apk", "curl,git"
	 Suggestions: [player, picoline, pillager]
tests/apt_get_test.rs:4:17      - Unknown word (picolayer)  -- use common::run_picolayer;
	 Suggestions: [player, picoline, pillager]
tests/apt_get_test.rs:33:22     - Unknown word (picolayer)  -- let output = run_picolayer(&["apt-get", "file"
	 Suggestions: [player, picoline, pillager]
tests/apt_get_test.rs:51:22     - Unknown word (ppas)       -- fn test_apt_get_with_ppas() {
	 Suggestions: [paps, pas, peas, ppam, pyas]
tests/apt_get_test.rs:63:22     - Unknown word (picolayer)  -- let output = run_picolayer(&["apt-get", "file"
	 Suggestions: [player, picoline, pillager]
tests/apt_get_test.rs:63:56     - Unknown word (ppas)       -- apt-get", "file", "--ppas", "ppa:git-core/ppa
	 Suggestions: [paps, pas, peas, ppam, pyas]
tests/apt_get_test.rs:94:22     - Unknown word (picolayer)  -- let output = run_picolayer(&["apt-get", "-s",
	 Suggestions: [player, picoline, pillager]
tests/apt_get_test.rs:136:22    - Unknown word (picolayer)  -- let output = run_picolayer(&["apt-get", "update
	 Suggestions: [player, picoline, pillager]
tests/brew_test.rs:3:17      - Unknown word (picolayer)  -- use common::run_picolayer;
	 Suggestions: [player, picoline, pillager]
tests/brew_test.rs:22:22     - Unknown word (picolayer)  -- let output = run_picolayer(&["brew", "jq"]);
	 Suggestions: [player, picoline, pillager]
tests/brew_test.rs:50:22     - Unknown word (picolayer)  -- let output = run_picolayer(&["brew", "jq,tree"
	 Suggestions: [player, picoline, pillager]
tests/common/mod.rs:6:17      - Unknown word (picolayer)  -- /// Path to the picolayer binary for testing
	 Suggestions: [player, picoline, pillager]
tests/common/mod.rs:8:11      - Unknown word (PICOLAYER)  -- pub const PICOLAYER_BIN: &str = env!("CARGO
	 Suggestions: [PLAYER, PICOLINE, PILLAGER]
tests/common/mod.rs:8:53      - Unknown word (picolayer)  -- env!("CARGO_BIN_EXE_picolayer");
	 Suggestions: [player, picoline, pillager]
tests/common/mod.rs:10:9      - Unknown word (picolayer)  -- /// Run picolayer with the given arguments
	 Suggestions: [player, picoline, pillager]
tests/common/mod.rs:12:12     - Unknown word (picolayer)  -- pub fn run_picolayer(args: &[&str]) -> std
	 Suggestions: [player, picoline, pillager]
tests/common/mod.rs:13:18     - Unknown word (PICOLAYER)  -- Command::new(PICOLAYER_BIN)
	 Suggestions: [PLAYER, PICOLINE, PILLAGER]
tests/common/mod.rs:16:36     - Unknown word (picolayer)  -- expect("Failed to execute picolayer")
	 Suggestions: [player, picoline, pillager]
tests/gh_release_test.rs:3:75      - Unknown word (picolayer)  -- transient_error, run_picolayer};
	 Suggestions: [player, picoline, pillager]
tests/gh_release_test.rs:7:9       - Unknown word (pkgx)       -- fn test_pkgx_github_release_installation
	 Suggestions: [pkgs, pkg, paxg, page, payx]
tests/gh_release_test.rs:11:22     - Unknown word (picolayer)  -- let output = run_picolayer(&[
	 Suggestions: [player, picoline, pillager]
tests/gh_release_test.rs:13:10     - Unknown word (pkgxdev)    -- "pkgxdev/pkgx",
	 Suggestions: [pkgadd]
tests/gh_release_test.rs:13:18     - Unknown word (pkgx)       -- "pkgxdev/pkgx",
	 Suggestions: [pkgs, pkg, paxg, page, payx]
tests/gh_release_test.rs:14:10     - Unknown word (pkgx)       -- "pkgx",
	 Suggestions: [pkgs, pkg, paxg, page, payx]
tests/gh_release_test.rs:36:10     - Unknown word (pkgx)       -- "pkgx installation failed
	 Suggestions: [pkgs, pkg, paxg, page, payx]
tests/gh_release_test.rs:40:35     - Unknown word (pkgx)       -- binary_path = format!("{}/pkgx", bin_location);
	 Suggestions: [pkgs, pkg, paxg, page, payx]
tests/gh_release_test.rs:54:9      - Unknown word (lazygit)    -- fn test_lazygit_specific_version_installation
	 Suggestions: [lait, lazy, lazio, layout, lazuli]
tests/gh_release_test.rs:58:22     - Unknown word (picolayer)  -- let output = run_picolayer(&[
	 Suggestions: [player, picoline, pillager]
tests/gh_release_test.rs:60:10     - Unknown word (jesseduffield) -- "jesseduffield/lazygit",
	 Suggestions: []
tests/gh_release_test.rs:60:24     - Unknown word (lazygit)       -- "jesseduffield/lazygit",
	 Suggestions: [lait, lazy, lazio, layout, lazuli]
tests/gh_release_test.rs:61:10     - Unknown word (lazygit)       -- "lazygit",
	 Suggestions: [lait, lazy, lazio, layout, lazuli]
tests/gh_release_test.rs:83:10     - Unknown word (lazygit)       -- "lazygit v0.54.0 installation
	 Suggestions: [lait, lazy, lazio, layout, lazuli]
tests/gh_release_test.rs:87:35     - Unknown word (lazygit)       -- binary_path = format!("{}/lazygit", bin_location);
	 Suggestions: [lait, lazy, lazio, layout, lazuli]
tests/gh_release_test.rs:105:22    - Unknown word (picolayer)     -- let output = run_picolayer(&[
	 Suggestions: [player, picoline, pillager]
tests/gh_release_test.rs:107:10    - Unknown word (jesseduffield) -- "jesseduffield/lazygit",
	 Suggestions: []
tests/gh_release_test.rs:154:22    - Unknown word (picolayer)     -- let output = run_picolayer(&[
	 Suggestions: [player, picoline, pillager]
tests/gh_release_test.rs:156:10    - Unknown word (pkgxdev)       -- "pkgxdev/pkgx",
	 Suggestions: [pkgadd]
tests/gh_release_test.rs:203:10    - Unknown word (pkgxdev)       -- "pkgxdev/pkgx",
	 Suggestions: [pkgadd]
tests/gh_release_test.rs:251:10    - Unknown word (pkgxdev)       -- "pkgxdev/pkgx",
	 Suggestions: [pkgadd]
tests/gh_release_test.rs:290:10    - Unknown word (pkgxdev)       -- "pkgxdev/pkgx",
	 Suggestions: [pkgadd]
tests/main_test.rs:3:17      - Unknown word (picolayer)  -- use common::run_picolayer;
	 Suggestions: [player, picoline, pillager]
tests/main_test.rs:7:22      - Unknown word (picolayer)  -- let output = run_picolayer(&["--help"]);
	 Suggestions: [player, picoline, pillager]
tests/main_test.rs:10:30     - Unknown word (picolayer)  -- assert!(stdout.contains("picolayer"));
	 Suggestions: [player, picoline, pillager]
tests/main_test.rs:16:22     - Unknown word (picolayer)  -- let output = run_picolayer(&["--version"]);
	 Suggestions: [player, picoline, pillager]
tests/main_test.rs:19:30     - Unknown word (picolayer)  -- assert!(stdout.contains("picolayer"));
	 Suggestions: [player, picoline, pillager]
tests/run_test.rs:3:33      - Unknown word (picolayer)  -- {binary_exists, run_picolayer};
	 Suggestions: [player, picoline, pillager]
tests/run_test.rs:8:9       - Unknown word (picolayer)  -- fn test_picolayer_run_python_version(
	 Suggestions: [player, picoline, pillager]
tests/run_test.rs:9:22      - Unknown word (picolayer)  -- let output = run_picolayer(&["run", "python@3.
	 Suggestions: [player, picoline, pillager]
tests/run_test.rs:16:9      - Unknown word (picolayer)  -- fn test_picolayer_run_node_version()
	 Suggestions: [player, picoline, pillager]
tests/run_test.rs:17:22     - Unknown word (picolayer)  -- let output = run_picolayer(&["run", "node@18",
	 Suggestions: [player, picoline, pillager]
tests/run_test.rs:181:34    - Unknown word (pkgx)       -- picolayer_run_with_force_pkgx() {
	 Suggestions: [pkgs, pkg, paxg, page, payx]
tests/run_test.rs:182:76    - Unknown word (pkgx)       -- , "world", "--force-pkgx"]);
	 Suggestions: [pkgs, pkg, paxg, page, payx]
tests/run_test.rs:193:30    - Unknown word (rustc)      -- assert!(stdout.contains("rustc 1.70"));
	 Suggestions: [rust, rusts, rusty, rustic, Rusty]
tests/run_test.rs:229:9     - Unknown word (pkgx)       -- fn test_pkgx_xz_installation_end
	 Suggestions: [pkgs, pkg, paxg, page, payx]
tests/run_test.rs:241:14    - Unknown word (pkgxdev)    -- "pkgxdev/pkgx",
	 Suggestions: [pkgadd]
tests/run_test.rs:241:22    - Unknown word (pkgx)       -- "pkgxdev/pkgx",
	 Suggestions: [pkgs, pkg, paxg, page, payx]
tests/run_test.rs:242:14    - Unknown word (pkgx)       -- "pkgx",
	 Suggestions: [pkgs, pkg, paxg, page, payx]
CSpell: Files checked: 27, Issues found: 182 in 20 files.


You can skip this misspellings by defining the following .cspell.json file at the root of your repository
Of course, please correct real typos before :)

{
    "version": "0.2",
    "language": "en",
    "ignorePaths": [
        "**/node_modules/**",
        "**/vscode-extension/**",
        "**/.git/**",
        "**/.pnpm-lock.json",
        ".vscode",
        "package-lock.json",
        "megalinter-reports"
    ],
    "words": [
        "Buildx",
        "Clippy",
        "Distro",
        "GOPATH",
        "GOROOT",
        "PICOLAYER",
        "Picolayer",
        "armv",
        "buildx",
        "clippy",
        "distro",
        "dtolnay",
        "elif",
        "flate",
        "gofmt",
        "indicatif",
        "javac",
        "jesseduffield",
        "lazygit",
        "libpkgx",
        "lzma",
        "nanolayer",
        "picolayer",
        "pkgm",
        "pkgx",
        "pkgxdev",
        "ppas",
        "pyproject",
        "reqs",
        "reqwest",
        "rusqlite",
        "rustc",
        "rustls",
        "serde",
        "shellenv",
        "skevetter",
        "softprops",
        "walkdir",
        "webpki"
    ]
}


You can also copy-paste megalinter-reports/.cspell.json at the root of your repository
```

</details>

<details>
<summary>⚠️ EDITORCONFIG / editorconfig-checker - 1 error</summary>

```
src/brew.rs:
	17: Wrong amount of left-padding spaces(want multiple of 4)
src/run.rs:
	523: Wrong amount of left-padding spaces(want multiple of 4)

2 errors found
```

</details>

<details>
<summary>⚠️ REPOSITORY / grype - 1 error</summary>

```
[0000]  WARN no explicit name and version provided for directory source, deriving artifact ID from the given path (which is not ideal)
NAME                       INSTALLED  FIXED IN  TYPE           VULNERABILITY        SEVERITY  EPSS  RISK  
actions/download-artifact  v4         4.1.3     github-action  GHSA-cxww-7g56-2vh6  High      N/A   N/A
[0037] ERROR discovered vulnerabilities at or above the severity threshold
```

</details>

<details>
<summary>⚠️ DOCKERFILE / hadolint - 1 error</summary>

```
Dockerfile:6 DL3008 warning: Pin versions in apt get install. Instead of `apt-get install <package>` use `apt-get install <package>=<version>`
```

</details>

<details>
<summary>⚠️ COPYPASTE / jscpd - 18 errors</summary>

```
Clone found (rust):
 - tests/gh_release_test.rs [63:10 - 83:42] (20 lines, 127 tokens)
   tests/gh_release_test.rs [16:9 - 36:31]

Clone found (rust):
 - tests/gh_release_test.rs [101:34 - 110:9] (9 lines, 77 tokens)
   tests/gh_release_test.rs [54:43 - 63:10]

Clone found (rust):
 - tests/gh_release_test.rs [113:13 - 131:55] (18 lines, 119 tokens)
   tests/gh_release_test.rs [18:13 - 36:31]

Clone found (rust):
 - tests/gh_release_test.rs [210:7 - 228:43] (18 lines, 119 tokens)
   tests/gh_release_test.rs [18:13 - 36:31]

Clone found (rust):
 - tests/gh_release_test.rs [245:37 - 254:9] (9 lines, 77 tokens)
   tests/gh_release_test.rs [7:38 - 16:9]

Clone found (rust):
 - tests/gh_release_test.rs [279:47 - 288:14] (9 lines, 107 tokens)
   tests/run_test.rs [229:37 - 238:4]

Clone found (rust):
 - tests/gh_release_test.rs [317:40 - 331:15] (14 lines, 129 tokens)
   tests/run_test.rs [229:37 - 293:9]

Clone found (rust):
 - tests/gh_release_test.rs [356:12 - 381:2] (25 lines, 183 tokens)
   tests/gh_release_test.rs [281:2 - 307:4]

Clone found (rust):
 - tests/gh_release_test.rs [392:42 - 401:61] (9 lines, 101 tokens)
   tests/run_test.rs [229:37 - 238:4]

Clone found (rust):
 - tests/gh_release_test.rs [402:5 - 422:2] (20 lines, 133 tokens)
   tests/gh_release_test.rs [288:5 - 383:3]

Clone found (rust):
 - tests/gh_release_test.rs [435:5 - 443:6] (8 lines, 97 tokens)
   tests/run_test.rs [230:5 - 238:7]

Clone found (rust):
 - tests/gh_release_test.rs [469:5 - 499:2] (30 lines, 229 tokens)
   tests/run_test.rs [225:2 - 307:4]

Clone found (rust):
 - tests/gh_release_test.rs [515:19 - 539:46] (24 lines, 208 tokens)
   tests/run_test.rs [229:37 - 303:39]

Clone found (rust):
 - tests/gh_release_test.rs [549:38 - 562:43] (13 lines, 93 tokens)
   tests/gh_release_test.rs [7:38 - 258:27]

Clone found (rust):
 - src/run.rs [542:2 - 552:38] (10 lines, 80 tokens)
   src/run.rs [314:2 - 324:41]

Clone found (rust):
 - src/gh_release.rs [234:19 - 251:9] (17 lines, 224 tokens)
   src/gh_release.rs [199:34 - 216:30]

Clone found (rust):
 - src/gh_release.rs [445:9 - 454:3] (9 lines, 93 tokens)
   src/gh_release.rs [420:21 - 429:2]

Clone found (rust):
 - src/gh_release.rs [566:5 - 583:18] (17 lines, 138 tokens)
   src/gh_release.rs [303:5 - 320:17]

┌──────────┬────────────────┬─────────────┬──────────────┬──────────────┬──────────────────┬───────────────────┐
│ Format   │ Files analyzed │ Total lines │ Total tokens │ Clones found │ Duplicated lines │ Duplicated tokens │
├──────────┼────────────────┼─────────────┼──────────────┼──────────────┼──────────────────┼───────────────────┤
│ rust     │ 16             │ 3199        │ 25545        │ 18           │ 279 (8.72%)      │ 2334 (9.14%)      │
├──────────┼────────────────┼─────────────┼──────────────┼──────────────┼──────────────────┼───────────────────┤
│ bash     │ 1              │ 13          │ 66           │ 0            │ 0 (0%)           │ 0 (0%)            │
├──────────┼────────────────┼─────────────┼──────────────┼──────────────┼──────────────────┼───────────────────┤
│ markdown │ 1              │ 26          │ 179          │ 0            │ 0 (0%)           │ 0 (0%)            │
├──────────┼────────────────┼─────────────┼──────────────┼──────────────┼──────────────────┼───────────────────┤
│ Total:   │ 18             │ 3238        │ 25790        │ 18           │ 279 (8.62%)      │ 2334 (9.05%)      │
└──────────┴────────────────┴─────────────┴──────────────┴──────────────┴──────────────────┴───────────────────┘
Found 18 clones.
HTML report saved to megalinter-reports/copy-paste/html/
ERROR: jscpd found too many duplicates (8.62%) over threshold (0%)
Error: ERROR: jscpd found too many duplicates (8.62%) over threshold (0%)
    at ThresholdReporter.report (/node-deps/node_modules/@jscpd/finder/dist/index.js:612:13)
    at /node-deps/node_modules/@jscpd/finder/dist/index.js:110:18
    at Array.forEach (<anonymous>)
    at /node-deps/node_modules/@jscpd/finder/dist/index.js:109:22
    at async /node-deps/node_modules/jscpd/dist/jscpd.js:351:5
```

</details>

<details>
<summary>⚠️ REPOSITORY / kics - 16 errors</summary>

```
MLLLLLM             MLLLLLLLLL   LLLLLLL             KLLLLLLLLLLLLLLLL       LLLLLLLLLLLLLLLLLLLLLLL 
   MMMMMMM           MMMMMMMMMML    MMMMMMMK       LMMMMMMMMMMMMMMMMMMMML   KLMMMMMMMMMMMMMMMMMMMMMMMMM 
   MMMMMMM         MMMMMMMMML       MMMMMMMK     LMMMMMMMMMMMMMMMMMMMMMML  LMMMMMMMMMMMMMMMMMMMMMMMMMMM 
   MMMMMMM      MMMMMMMMMML         MMMMMMMK   LMMMMMMMMMMMMMMMMMMMMMMMML LMMMMMMMMMMMMMMMMMMMMMMMMMMMM 
   MMMMMMM    LMMMMMMMMML           MMMMMMMK  LMMMMMMMMMLLMLLLLLLLLLLLLLL LMMMMMMMLLLLLLLLLLLLLLLLLLLLM 
   MMMMMMM  MMMMMMMMMLM             MMMMMMMK LMMMMMMMM                    LMMMMMML                      
   MMMMMMMLMMMMMMMML                MMMMMMMK MMMMMMML                     LMMMMMMMMLLLLLLLLLLLLLMLL     
   MMMMMMMMMMMMMMMM                 MMMMMMMK MMMMMML                       LMMMMMMMMMMMMMMMMMMMMMMMMML  
   MMMMMMMMMMMMMMMMMM               MMMMMMMK MMMMMMM                         LMMMMMMMMMMMMMMMMMMMMMMMML 
   MMMMMMM KLMMMMMMMMML             MMMMMMMK LMMMMMMM                                          MMMMMMMML
   MMMMMMM    LMMMMMMMMMM           MMMMMMMK LMMMMMMMMLL                                        MMMMMMML
   MMMMMMM      LMMMMMMMMMLL        MMMMMMMK  LMMMMMMMMMMMMMMMMMMMMMMMMML LLLLLLLLLLLLLLLLLLLLMMMMMMMMMM
   MMMMMMM        MMMMMMMMMMML      MMMMMMMK   MMMMMMMMMMMMMMMMMMMMMMMMML LMMMMMMMMMMMMMMMMMMMMMMMMMMMM 
   MMMMMMM          LLMMMMMMMMML    MMMMMMMK     LLMMMMMMMMMMMMMMMMMMMMML LMMMMMMMMMMMMMMMMMMMMMMMMMML  
   MMMMMMM             MMMMMMMMMML  MMMMMMMK         KLMMMMMMMMMMMMMMMMML LMMMMMMMMMMMMMMMMMMMMMMMLK    
                                                                                                            
                                                                                                                                                                                                                                                                                                                        


Scanning with Keeping Infrastructure as Code Secure v2.1.13





Unpinned Actions Full Length Commit SHA, Severity: LOW, Results: 13
Description: Pinning an action to a full length commit SHA is currently the only way to use an action as an immutable release. Pinning to a particular SHA helps mitigate the risk of a bad actor adding a backdoor to the action's repository, as they would need to generate a SHA-1 collision for a valid Git object payload. When selecting a SHA, you should verify it is from the action's repository and not a repository fork.
Platform: CICD
CWE: 829
Learn more about this vulnerability: https://docs.kics.io/latest/queries/cicd-queries/555ab8f9-2001-455e-a077-f2d0f41e2fb9

	[1]: .github/workflows/docker.yml:59

		058:       - name: Build and push Docker image
		059:         uses: docker/build-push-action@v5
		060:         with:


	[2]: .github/workflows/release-nightly.yml:119

		118:       - name: Create Nightly Release
		119:         uses: softprops/action-gh-release@v2
		120:         with:


	[3]: .github/workflows/lint.yml:77

		076:         if: steps.ml.outputs.has_updated_sources == 1 && (env.APPLY_FIXES_EVENT == 'all' || env.APPLY_FIXES_EVENT == github.event_name) && env.APPLY_FIXES_MODE == 'pull_request' && (github.event_name == 'push' || github.event.pull_request.head.repo.full_n
		077:         uses: peter-evans/create-pull-request@v7
		078:         with:


	[4]: .github/workflows/lint.yml:96

		095:         if: steps.ml.outputs.has_updated_sources == 1 && (env.APPLY_FIXES_EVENT == 'all' || env.APPLY_FIXES_EVENT == github.event_name) && env.APPLY_FIXES_MODE == 'commit' && github.ref != 'refs/heads/main' && (github.event_name == 'push' || github.event.
		096:         uses: stefanzweifel/git-auto-commit-action@v6
		097:         with:


	[5]: .github/workflows/pre-commit.yml:16

		015: 
		016:       - uses: biomejs/setup-biome@v2
		017: 


	[6]: .github/workflows/devcontainer.yml:31

		030: 
		031:       - uses: devcontainers/ci@v0.3
		032:         with:


	[7]: .github/workflows/docker.yml:28

		027:       - name: Log in to Container Registry
		028:         uses: docker/login-action@v3
		029:         with:


	[8]: .github/workflows/docker.yml:25

		024:       - name: Set up Docker Buildx
		025:         uses: docker/setup-buildx-action@v3
		026: 


	[9]: .github/workflows/docker.yml:35

		034:       - name: Setup Rust
		035:         uses: dtolnay/rust-toolchain@stable
		036:         with:


	[10]: .github/workflows/lint.yml:58

		057:         # More info at https://megalinter.io/flavors/
		058:         uses: oxsecurity/megalinter@v9
		059:         env:


	[11]: .github/workflows/docker.yml:47

		046:         id: meta
		047:         uses: docker/metadata-action@v5
		048:         with:


	[12]: .github/workflows/release-nightly.yml:35

		034: 
		035:       - uses: dtolnay/rust-toolchain@stable
		036:         with:


	[13]: .github/workflows/release.yml:119

		118:       - name: Create Release
		119:         uses: softprops/action-gh-release@v2
		120:         with:


Healthcheck Instruction Missing, Severity: LOW, Results: 1
Description: Ensure that HEALTHCHECK is being used. The HEALTHCHECK instruction tells Docker how to test a container to check that it is still working
Platform: Dockerfile
CWE: 710
Learn more about this vulnerability: https://docs.kics.io/latest/queries/dockerfile-queries/b03a748a-542d-44f4-bb86-9199ab4fd2d5

	[1]: Dockerfile:1

		001: FROM debian:12-slim
		002: 
		003: ARG TARGETARCH


Apt Get Install Pin Version Not Defined, Severity: MEDIUM, Results: 1
Description: When installing a package, its pin version should be defined
Platform: Dockerfile
CWE: 1357
Learn more about this vulnerability: https://docs.kics.io/latest/queries/dockerfile-queries/965a08d7-ef86-4f14-8792-4a3b2098937e

	[1]: Dockerfile:6

		005: 
		006: RUN apt-get update && \
		007:     apt-get install -y --no-install-recommends ca-certificates && \


Missing User Instruction, Severity: HIGH, Results: 1
Description: A user should be specified in the dockerfile, otherwise the image will run as root
Platform: Dockerfile
CWE: 250
Learn more about this vulnerability: https://docs.kics.io/latest/queries/dockerfile-queries/fd54f200-402c-4333-a5a4-36ef6709af2f

	[1]: Dockerfile:1

		001: FROM debian:12-slim
		002: 
		003: ARG TARGETARCH



Results Summary:
CRITICAL: 0
HIGH: 1
MEDIUM: 1
LOW: 14
INFO: 0
TOTAL: 16

A new version 'v2.1.14' of KICS is available, please consider updating
```

</details>

<details>
<summary>⚠️ REPOSITORY / trivy - 1 error</summary>

```
2025-10-07T03:56:36Z	INFO	[vulndb] Need to update DB
2025-10-07T03:56:36Z	INFO	[vulndb] Downloading vulnerability DB...
2025-10-07T03:56:36Z	INFO	[vulndb] Downloading artifact...	repo="mirror.gcr.io/aquasec/trivy-db:2"
5.91 MiB / 72.06 MiB [----->_________________________________________________________] 8.20% ? p/s ?28.51 MiB / 72.06 MiB [------------------------>____________________________________] 39.56% ? p/s ?53.73 MiB / 72.06 MiB [--------------------------------------------->_______________] 74.56% ? p/s ?72.06 MiB / 72.06 MiB [--------------------------------------------->] 100.00% 110.12 MiB p/s ETA 0s72.06 MiB / 72.06 MiB [--------------------------------------------->] 100.00% 110.12 MiB p/s ETA 0s72.06 MiB / 72.06 MiB [--------------------------------------------->] 100.00% 110.12 MiB p/s ETA 0s72.06 MiB / 72.06 MiB [--------------------------------------------->] 100.00% 103.02 MiB p/s ETA 0s72.06 MiB / 72.06 MiB [--------------------------------------------->] 100.00% 103.02 MiB p/s ETA 0s72.06 MiB / 72.06 MiB [--------------------------------------------->] 100.00% 103.02 MiB p/s ETA 0s72.06 MiB / 72.06 MiB [---------------------------------------------->] 100.00% 96.37 MiB p/s ETA 0s72.06 MiB / 72.06 MiB [---------------------------------------------->] 100.00% 96.37 MiB p/s ETA 0s72.06 MiB / 72.06 MiB [---------------------------------------------->] 100.00% 96.37 MiB p/s ETA 0s72.06 MiB / 72.06 MiB [---------------------------------------------->] 100.00% 90.16 MiB p/s ETA 0s72.06 MiB / 72.06 MiB [---------------------------------------------->] 100.00% 90.16 MiB p/s ETA 0s72.06 MiB / 72.06 MiB [---------------------------------------------->] 100.00% 90.16 MiB p/s ETA 0s72.06 MiB / 72.06 MiB [---------------------------------------------->] 100.00% 84.34 MiB p/s ETA 0s72.06 MiB / 72.06 MiB [---------------------------------------------->] 100.00% 84.34 MiB p/s ETA 0s72.06 MiB / 72.06 MiB [---------------------------------------------->] 100.00% 84.34 MiB p/s ETA 0s72.06 MiB / 72.06 MiB [---------------------------------------------->] 100.00% 78.90 MiB p/s ETA 0s72.06 MiB / 72.06 MiB [---------------------------------------------->] 100.00% 78.90 MiB p/s ETA 0s72.06 MiB / 72.06 MiB [---------------------------------------------->] 100.00% 78.90 MiB p/s ETA 0s72.06 MiB / 72.06 MiB [---------------------------------------------->] 100.00% 73.81 MiB p/s ETA 0s72.06 MiB / 72.06 MiB [---------------------------------------------->] 100.00% 73.81 MiB p/s ETA 0s72.06 MiB / 72.06 MiB [---------------------------------------------->] 100.00% 73.81 MiB p/s ETA 0s72.06 MiB / 72.06 MiB [---------------------------------------------->] 100.00% 69.05 MiB p/s ETA 0s72.06 MiB / 72.06 MiB [---------------------------------------------->] 100.00% 69.05 MiB p/s ETA 0s72.06 MiB / 72.06 MiB [---------------------------------------------->] 100.00% 69.05 MiB p/s ETA 0s72.06 MiB / 72.06 MiB [-------------------------------------------------] 100.00% 13.73 MiB p/s 5.4s2025-10-07T03:56:42Z	INFO	[vulndb] Artifact successfully downloaded	repo="mirror.gcr.io/aquasec/trivy-db:2"
2025-10-07T03:56:42Z	INFO	[vuln] Vulnerability scanning is enabled
2025-10-07T03:56:42Z	INFO	[misconfig] Misconfiguration scanning is enabled
2025-10-07T03:56:42Z	INFO	[misconfig] Need to update the checks bundle
2025-10-07T03:56:42Z	INFO	[misconfig] Downloading the checks bundle...
165.20 KiB / 165.20 KiB [------------------------------------------------------] 100.00% ? p/s 100ms2025-10-07T03:56:45Z	INFO	[npm] To collect the license information of packages, "npm install" needs to be performed beforehand	dir="node_modules"
2025-10-07T03:56:45Z	INFO	Suppressing dependencies for development and testing. To display them, try the '--include-dev-deps' flag.
2025-10-07T03:56:45Z	INFO	Number of language-specific files	num=2
2025-10-07T03:56:45Z	INFO	[cargo] Detecting vulnerabilities...
2025-10-07T03:56:45Z	INFO	Detected config files	num=4

Report Summary

┌─────────────────────────────────┬────────────┬─────────────────┬───────────────────┐
│             Target              │    Type    │ Vulnerabilities │ Misconfigurations │
├─────────────────────────────────┼────────────┼─────────────────┼───────────────────┤
│ Cargo.lock                      │   cargo    │        0        │         -         │
├─────────────────────────────────┼────────────┼─────────────────┼───────────────────┤
│ Dockerfile                      │ dockerfile │        -        │         2         │
├─────────────────────────────────┼────────────┼─────────────────┼───────────────────┤
│ examples/Dockerfile.alpine      │ dockerfile │        -        │         2         │
├─────────────────────────────────┼────────────┼─────────────────┼───────────────────┤
│ examples/Dockerfile.traditional │ dockerfile │        -        │         3         │
├─────────────────────────────────┼────────────┼─────────────────┼───────────────────┤
│ examples/Dockerfile.ubuntu      │ dockerfile │        -        │         2         │
└─────────────────────────────────┴────────────┴─────────────────┴───────────────────┘
Legend:
- '-': Not scanned
- '0': Clean (no security findings detected)


Dockerfile (dockerfile)
=======================
Tests: 27 (SUCCESSES: 25, FAILURES: 2)
Failures: 2 (UNKNOWN: 0, LOW: 1, MEDIUM: 0, HIGH: 1, CRITICAL: 0)

AVD-DS-0002 (HIGH): Specify at least 1 USER command in Dockerfile with non-root user as argument
════════════════════════════════════════
Running containers with 'root' user can lead to a container escape situation. It is a best practice to run containers as non-root users, which can be done by adding a 'USER' statement to the Dockerfile.

See https://avd.aquasec.com/misconfig/ds002
────────────────────────────────────────


AVD-DS-0026 (LOW): Add HEALTHCHECK instruction in your Dockerfile
════════════════════════════════════════
You should add HEALTHCHECK instruction in your docker container images to perform the health check on running containers.

See https://avd.aquasec.com/misconfig/ds026
────────────────────────────────────────



examples/Dockerfile.alpine (dockerfile)
=======================================
Tests: 27 (SUCCESSES: 25, FAILURES: 2)
Failures: 2 (UNKNOWN: 0, LOW: 1, MEDIUM: 0, HIGH: 1, CRITICAL: 0)

AVD-DS-0002 (HIGH): Specify at least 1 USER command in Dockerfile with non-root user as argument
════════════════════════════════════════
Running containers with 'root' user can lead to a container escape situation. It is a best practice to run containers as non-root users, which can be done by adding a 'USER' statement to the Dockerfile.

See https://avd.aquasec.com/misconfig/ds002
────────────────────────────────────────


AVD-DS-0026 (LOW): Add HEALTHCHECK instruction in your Dockerfile
════════════════════════════════════════
You should add HEALTHCHECK instruction in your docker container images to perform the health check on running containers.

See https://avd.aquasec.com/misconfig/ds026
────────────────────────────────────────



examples/Dockerfile.traditional (dockerfile)
============================================
Tests: 27 (SUCCESSES: 24, FAILURES: 3)
Failures: 3 (UNKNOWN: 0, LOW: 1, MEDIUM: 0, HIGH: 2, CRITICAL: 0)

AVD-DS-0002 (HIGH): Specify at least 1 USER command in Dockerfile with non-root user as argument
════════════════════════════════════════
Running containers with 'root' user can lead to a container escape situation. It is a best practice to run containers as non-root users, which can be done by adding a 'USER' statement to the Dockerfile.

See https://avd.aquasec.com/misconfig/ds002
────────────────────────────────────────


AVD-DS-0026 (LOW): Add HEALTHCHECK instruction in your Dockerfile
════════════════════════════════════════
You should add HEALTHCHECK instruction in your docker container images to perform the health check on running containers.

See https://avd.aquasec.com/misconfig/ds026
────────────────────────────────────────


AVD-DS-0029 (HIGH): '--no-install-recommends' flag is missed: 'apt-get update && apt-get install -y curl git htop'
════════════════════════════════════════
'apt-get' install should use '--no-install-recommends' to minimize image size.

See https://avd.aquasec.com/misconfig/ds029
────────────────────────────────────────
 examples/Dockerfile.traditional:5
────────────────────────────────────────
   5 [ RUN apt-get update && apt-get install -y curl git htop
────────────────────────────────────────



examples/Dockerfile.ubuntu (dockerfile)
=======================================
Tests: 27 (SUCCESSES: 25, FAILURES: 2)
Failures: 2 (UNKNOWN: 0, LOW: 1, MEDIUM: 0, HIGH: 1, CRITICAL: 0)

AVD-DS-0002 (HIGH): Specify at least 1 USER command in Dockerfile with non-root user as argument
════════════════════════════════════════
Running containers with 'root' user can lead to a container escape situation. It is a best practice to run containers as non-root users, which can be done by adding a 'USER' statement to the Dockerfile.

See https://avd.aquasec.com/misconfig/ds002
────────────────────────────────────────


AVD-DS-0026 (LOW): Add HEALTHCHECK instruction in your Dockerfile
════════════════════════════════════════
You should add HEALTHCHECK instruction in your docker container images to perform the health check on running containers.

See https://avd.aquasec.com/misconfig/ds026
────────────────────────────────────────



📣 Notices:
  - Version 0.67.0 of Trivy is now available, current version is 0.66.0

To suppress version checks, run Trivy scans with the --skip-version-check flag
```

</details>

See detailed reports in MegaLinter artifacts
_Set `VALIDATE_ALL_CODEBASE: true` in mega-linter.yml to validate all sources, not only the diff_

[![MegaLinter is graciously provided by OX Security](https://raw.githubusercontent.com/oxsecurity/megalinter/main/docs/assets/images/ox-banner.png)](https://www.ox.security/?ref=megalinter)