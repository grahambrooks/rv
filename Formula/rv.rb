class Rv < Formula
  desc "Generate SVG visualizations of directory structures"
  homepage "https://github.com/grahambrooks/rv"
  version "2026.7.3"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/grahambrooks/rv/archive/refs/tags/v2026.8.1.tar.gz"
      sha256 "f417c55b7a67d27a2bdf50f557cf2b69e7d2f69cc9d0fdb767e97f0a50403d4a"
    end
    on_intel do
      odie "Intel Mac binaries are not provided. Run `cargo install --git https://github.com/grahambrooks/rv --locked` to build from source."
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/grahambrooks/rv/releases/download/v2026.7.3/rv-v2026.7.3-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "9933740c120d94f31fec73f0bef55399de18eb8100efb7087f2066177b487f36"
    end
    on_intel do
      url "https://github.com/grahambrooks/rv/releases/download/v2026.7.3/rv-v2026.7.3-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "e507bc9dc91625d22f7f03d1c43710d6846274ac8f8cbe24b731d346fd34e3f7"
    end
  end

  def install
    bin.install "rv"
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/rv --version")
  end
end
