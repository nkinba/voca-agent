class Spread < Formula
  desc "Headless TOEFL Vocabulary Builder for Developers"
  homepage "https://github.com/nkinba/voca-agent"
  url "https://github.com/nkinba/voca-agent/archive/refs/tags/v0.1.0.tar.gz"
  sha256 "PLACEHOLDER_SHA256"
  license "MIT"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args(path: "app")
  end

  test do
    assert_match "spread", shell_output("#{bin}/spread --help")
  end
end
