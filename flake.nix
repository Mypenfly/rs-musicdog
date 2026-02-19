{
  description = "Rust TUI Music Player Development Environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      rust-overlay,
      flake-utils,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        # 使用特定版本的Rust工具链
        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [
            "rust-src"
            "rust-analyzer"
            "clippy"
            "rustfmt"
          ];
        };

        # 音频相关的系统依赖
        audioLibs = with pkgs; [
          alsa-lib # Linux音频
          alsa-lib.dev
          pkg-config
        ];

        # TUI相关的依赖（可选，如果需要特定后端）
        tuiLibs = with pkgs; [
          # ratatui通常纯Rust，但如果有crossterm/termion依赖需要终端库
        ];

      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs =
            with pkgs;
            [
              # Rust工具链
              rustToolchain
              cargo-watch # 热重载开发
              cargo-edit # 依赖管理 (cargo add/rm)
              cargo-outdated # 检查过期依赖
              cargo-audit # 安全检查

              # 构建工具
              gnumake
              cmake

              # 音频处理依赖
              ffmpeg # 如果需要ffmpeg解码
              libmpg123 # MP3解码
              flac # FLAC支持

              # 开发工具
              gdb
              valgrind # 内存检查

              # 合并音频库
            ]
            ++ audioLibs;

          # 环境变量配置
          shellHook = ''
            echo "🎵 Rust Music Player Dev Environment"
            echo "Rust: $(rustc --version)"
            echo "Cargo: $(cargo --version)"

            # 确保pkg-config能找到alsa
            export PKG_CONFIG_PATH="${pkgs.alsa-lib.dev}/lib/pkgconfig:$PKG_CONFIG_PATH"

            # 音频设备权限提示
            echo ""
            echo "提示: 如果播放无声音，请检查用户是否在'audio'组:"
            echo "  sudo usermod -a -G audio $USER"

          '';

          # 动态链接库路径
          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath audioLibs;
        };

        # 可以添加package构建配置用于nix build
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "music-player";
          version = "0.1.0";
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;

          nativeBuildInputs = audioLibs;

          # 如果有需要patch的依赖
          postPatch = ''
            # 例如处理某些依赖的查找路径
          '';
        };
      }
    );
}
