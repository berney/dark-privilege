#echo $ErrorActionPreference
#$ErrorActionPreference = "Stop"

Set-PSDebug -Trace 1

# Install chocolatey, because it works
Invoke-WebRequest https://chocolatey.org/install.ps1 -UseBasicParsing | Invoke-Expression
$packageParameters = @(
    "--add Microsoft.VisualStudio.Workload.VCTools"
    #"--add Microsoft.VisualStudio.Component.Windows81SDK"
    #"--add Microsoft.VisualStudio.Component.Windows10SDK.10240"
    #"--add Microsoft.VisualStudio.Component.Windows10SDK.10586"
    #"--add Microsoft.VisualStudio.Component.Windows10SDK.14393"
    #"--add Microsoft.VisualStudio.Component.Windows10SDK.18362"
    #"--add Microsoft.VisualStudio.Component.Windows10SDK.19041"
    #"--add Microsoft.VisualStudio.Component.Windows10SDK.20348"
    #"--add Microsoft.VisualStudio.Component.Windows11SDK.22000"
    #"--add Microsoft.VisualStudio.Component.Windows11SDK.22621"
    #"--add Microsoft.VisualStudio.Component.Windows11SDK.26100"
    "--add Microsoft.VisualStudio.Component.VC.Tools.x86.x64"
    "--includeRecommended"
    "--includeOptional"
    "--noUpdateInstaller"
    # Passive shows some output, but shouldn't require interaction
    # Default mode is `--quiet`, which is silent
    #"--passive"
) -join " "
try {
	choco install visualstudio2022buildtools `
		--package-parameters $packageParameters `
		-y
	if ($LASTEXITCODE -ne 0) {
		throw "Chocolatey installation of MSVC Build Tools failed!"
	}
	Write-Output "Installation of MSVC Build Tools succeeded"
}
catch {
	Write-Output "An error occurred during installation of MSVC Build Tools: $_"
	exit 1
}
Invoke-WebRequest -Uri https://static.rust-lang.org/rustup/dist/x86_64-pc-windows-msvc/rustup-init.exe -OutFile rustup-init.exe
./rustup-init.exe -y
$env:PATH += ";$($env:USERPROFILE)\.cargo\bin"
rustup show
rustup toolchain list
rustup target list --installed
