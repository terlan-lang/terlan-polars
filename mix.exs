defmodule TerlanPolars.MixProject do
  use Mix.Project

  @version "0.1.0-dev"
  @source_url "https://github.com/terlan-lang/terlan-polars"

  def project do
    [
      app: :terlan_polars,
      version: @version,
      description: "External Polars DataFrame package for Terlan",
      package: package(),
      deps: []
    ]
  end

  def application do
    [
      extra_applications: []
    ]
  end

  defp package do
    [
      name: "terlan_polars",
      licenses: ["MIT"],
      links: %{
        "GitHub" => @source_url,
        "Terlan" => "https://github.com/terlan-lang/terlan",
        "Polars" => "https://pola.rs"
      },
      files: [
        "README.md",
        "ROADMAP.md",
        "LICENSE",
        "terlan.toml",
        "src",
        "bindings",
        "native",
        "test",
        "examples"
      ]
    ]
  end
end
