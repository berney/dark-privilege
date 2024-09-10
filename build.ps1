#echo $ErrorActionPreference
#$ErrorActionPreference = "Stop"

Set-PSDebug -Trace 1

echo $env:PATH
$env:PATH += ";$($env:USERPROFILE)\.cargo\bin"
cargo build
cargo build --release
cargo install --path .
dark-privilege
