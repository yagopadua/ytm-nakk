<#
.SYNOPSIS
Signs ytm-rs binaries with a local, self-signed code-signing certificate.

.DESCRIPTION
Creates (once) a code-signing certificate in the current user's store, exports its
public part to installer\signing\ytm-rs-local.cer and signs the files given.

The certificate is only trusted on machines where it is installed as a trusted
root/publisher, so this is for local builds, not for public releases. Use a real
OV/EV certificate for those.

.EXAMPLE
# Create the certificate and trust it on this account (accept the Windows prompt).
.\installer\sign-local.ps1 -Trust

.EXAMPLE
# Sign the app and the installer.
.\installer\sign-local.ps1 ..\target\release\ytm-rs.exe ..\installer\Output\ytm-rs-setup-0.2.27.exe
#>
param(
    [switch]$Trust,
    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]]$Files
)

$ErrorActionPreference = 'Stop'
$subject = 'CN=ytm-rs Local Build, O=ytm-rs'
$dir = Join-Path $PSScriptRoot 'signing'
$cer = Join-Path $dir 'ytm-rs-local.cer'

New-Item -ItemType Directory -Force $dir | Out-Null

$cert = Get-ChildItem Cert:\CurrentUser\My |
    Where-Object { $_.Subject -eq $subject -and $_.HasPrivateKey } |
    Sort-Object NotAfter -Descending |
    Select-Object -First 1
if (-not $cert) {
    $cert = New-SelfSignedCertificate `
        -Type CodeSigningCert `
        -Subject $subject `
        -FriendlyName 'ytm-rs local code signing' `
        -CertStoreLocation Cert:\CurrentUser\My `
        -NotAfter (Get-Date).AddYears(5) `
        -KeyUsage DigitalSignature `
        -TextExtension @('2.5.29.37={text}1.3.6.1.5.5.7.3.3')
    Write-Host "Created certificate $($cert.Thumbprint)"
}

Export-Certificate -Cert $cert -FilePath $cer -Force | Out-Null
Write-Host "Public certificate: $cer"

if ($Trust) {
    foreach ($store in 'Root', 'TrustedPublisher') {
        $path = "Cert:\CurrentUser\$store"
        $already = Get-ChildItem $path | Where-Object Thumbprint -eq $cert.Thumbprint
        if (-not $already) {
            Import-Certificate -FilePath $cer -CertStoreLocation $path | Out-Null
            Write-Host "Trusted in CurrentUser\$store"
        } else {
            Write-Host "Already trusted in CurrentUser\$store"
        }
    }
}

if ($Files.Count -eq 0) {
    Write-Host 'Nothing to sign.'
    exit 0
}

$signtool = Get-ChildItem (Join-Path ${env:ProgramFiles(x86)} 'Windows Kits\10\bin\*\x64\signtool.exe') -ErrorAction SilentlyContinue |
    Sort-Object FullName -Descending |
    Select-Object -First 1
if (-not $signtool) {
    throw 'signtool.exe not found. Install the Windows SDK (Signing Tools) first.'
}

foreach ($file in $Files) {
    if (-not (Test-Path $file)) { throw "File not found: $file" }
    Write-Host "Signing $file"
    & $signtool.FullName sign /fd SHA256 /td SHA256 /tr http://timestamp.digicert.com /sha1 $cert.Thumbprint $file
    if ($LASTEXITCODE -ne 0) { throw "Signing failed: $file" }
    & $signtool.FullName verify /pa $file | Out-Null
    if ($LASTEXITCODE -ne 0) { throw "Signature verification failed: $file" }
}
