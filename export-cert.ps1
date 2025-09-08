$cert = Get-ChildItem -Path "cert:\CurrentUser\My" | Where-Object { $_.Subject -eq "CN=VMFort Development" }
$password = ConvertTo-SecureString -String "VMFort2024" -Force -AsPlainText
Export-PfxCertificate -Cert $cert -FilePath ".\VMFort.pfx" -Password $password
Write-Host "Certificate exported to VMFort.pfx with password: VMFort2024"