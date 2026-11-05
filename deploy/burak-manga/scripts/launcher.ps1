# Dosya Yolu: REPO_ROOT/deploy/burak-manga/scripts/launcher.ps1
# Amac: Yerel Docker Suwayomi kurulumunu Windows GUI uzerinden yonetir
# Modul - PowerShell
# Version: 0.2.0
# Aciklama: Baslat, durdur, durum, gunluk ve imzali katalog kisayolu
# Bagimli Oldugu Katman: View / Tool
[CmdletBinding()]
param([switch] $ValidateLayout)
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

if ([Environment]::OSVersion.Platform -ne [PlatformID]::Win32NT) { throw 'WINDOWS_REQUIRED' }
if ([System.Threading.Thread]::CurrentThread.ApartmentState -ne [System.Threading.ApartmentState]::STA) {
    throw 'STA_REQUIRED: Use Windows PowerShell -STA.'
}
$DeploymentRoot = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot '..')).Path
$ComposeFile = Join-Path $DeploymentRoot 'compose.yaml'
$StartScript = Join-Path $PSScriptRoot 'start-windows.ps1'
$CatalogUrl = 'https://raw.githubusercontent.com/b1glord/Suwayomi-Server-Extensions/repo/index.pb'
$WebUrl = 'http://127.0.0.1:4567'
foreach ($File in @($ComposeFile, $StartScript)) {
    if (-not (Test-Path -LiteralPath $File -PathType Leaf)) { throw "REQUIRED_FILE_MISSING: $File" }
}

Add-Type -AssemblyName PresentationFramework
Add-Type -AssemblyName PresentationCore
Add-Type -AssemblyName WindowsBase
$Xaml = @'
<Window xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation"
        xmlns:x="http://schemas.microsoft.com/winfx/2006/xaml"
        Title="Burak Manga Launcher v0.2.0" Width="870" Height="690"
        MinWidth="740" MinHeight="560" WindowStartupLocation="CenterScreen"
        Background="#101827" Foreground="#E6EDF7" FontFamily="Segoe UI">
  <Window.Resources>
    <Style TargetType="Button">
      <Setter Property="Height" Value="40"/>
      <Setter Property="MinWidth" Value="125"/>
      <Setter Property="Margin" Value="0,0,10,10"/>
      <Setter Property="Padding" Value="12,6"/>
      <Setter Property="Background" Value="#334155"/>
      <Setter Property="Foreground" Value="#F8FAFC"/>
      <Setter Property="BorderBrush" Value="#64748B"/>
      <Setter Property="FontSize" Value="13"/>
    </Style>
  </Window.Resources>
  <Grid Margin="24">
    <Grid.RowDefinitions>
      <RowDefinition Height="Auto"/>
      <RowDefinition Height="Auto"/>
      <RowDefinition Height="Auto"/>
      <RowDefinition Height="*"/>
      <RowDefinition Height="Auto"/>
    </Grid.RowDefinitions>
    <StackPanel Grid.Row="0" Margin="0,0,0,16">
      <TextBlock Text="BURAK MANGA" FontSize="28" FontWeight="Bold" Foreground="#F8FAFC"/>
      <TextBlock Text="Suwayomi sunucusu ve imzalı uzantı kataloğu" FontSize="14" Foreground="#94A3B8"/>
    </StackPanel>
    <Border Grid.Row="1" Background="#1E293B" CornerRadius="12" Padding="16" Margin="0,0,0,14">
      <StackPanel>
        <TextBlock Text="SUNUCU DURUMU" Foreground="#94A3B8" FontWeight="SemiBold"/>
        <TextBlock x:Name="StatusLabel" Text="Henüz kontrol edilmedi" FontSize="18" Margin="0,5,0,12"/>
        <WrapPanel>
          <Button x:Name="StartButton" Content="Başlat" Background="#0F766E"/>
          <Button x:Name="StopButton" Content="Durdur"/>
          <Button x:Name="RestartButton" Content="Yeniden Başlat"/>
          <Button x:Name="RefreshButton" Content="Durumu Yenile"/>
        </WrapPanel>
        <WrapPanel>
          <Button x:Name="WebButton" Content="WebUI'yi Aç" Background="#1D4ED8"/>
          <Button x:Name="LogsButton" Content="Günlükleri Göster"/>
        </WrapPanel>
      </StackPanel>
    </Border>
    <Border Grid.Row="2" Background="#1E293B" CornerRadius="12" Padding="16" Margin="0,0,0,14">
      <StackPanel>
        <TextBlock Text="UZANTI DEPOSU" Foreground="#94A3B8" FontWeight="SemiBold"/>
        <TextBlock Text="Rüya Manga + Merlin Scans" FontSize="17" Margin="0,5,0,8"/>
        <TextBox x:Name="CatalogAddress" IsReadOnly="True" BorderThickness="0"
                 Background="#0F172A" Foreground="#CBD5E1" Padding="9" FontSize="12" Margin="0,0,0,10"/>
        <WrapPanel>
          <Button x:Name="CopyButton" Content="Katalog Adresini Kopyala" MinWidth="195"/>
          <Button x:Name="CatalogButton" Content="Bağlantıyı Kontrol Et" MinWidth="175"/>
        </WrapPanel>
        <TextBlock Text="Kataloğu WebUI uzantı depolarına bir kez ekleyin. APK kurulumunu Suwayomi yapar."
                   Foreground="#94A3B8" TextWrapping="Wrap"/>
      </StackPanel>
    </Border>
    <Border Grid.Row="3" Background="#1E293B" CornerRadius="12" Padding="16">
      <DockPanel>
        <TextBlock DockPanel.Dock="Top" Text="İŞLEM ÇIKTISI" Foreground="#94A3B8" Margin="0,0,0,8"/>
        <TextBox x:Name="OutputLog" IsReadOnly="True" AcceptsReturn="True" TextWrapping="Wrap"
                 VerticalScrollBarVisibility="Auto" HorizontalScrollBarVisibility="Auto"
                 FontFamily="Consolas" FontSize="12" Padding="10" BorderThickness="0"
                 Background="#0F172A" Foreground="#CBD5E1"/>
      </DockPanel>
    </Border>
    <TextBlock Grid.Row="4" Margin="0,13,0,0" Foreground="#94A3B8" FontSize="12"
               Text="Veriler silinmez. Docker Desktop gereklidir. Başlatıcı kapatılınca sunucu çalışmaya devam eder."/>
  </Grid>
</Window>
'@
$Window = [Windows.Markup.XamlReader]::Parse($Xaml)
$Controls = @('StatusLabel','OutputLog','CatalogAddress','StartButton','StopButton',
              'RestartButton','RefreshButton','WebButton','LogsButton','CopyButton','CatalogButton')
foreach ($Control in $Controls) {
    if ($null -eq $Window.FindName($Control)) { throw "LAUNCHER_CONTROL_MISSING: $Control" }
}
if ($ValidateLayout) {
    Write-Output 'LAUNCHER_LAYOUT_VALID'
    return
}

$script:StatusLabel = $Window.FindName('StatusLabel')
$script:OutputLog = $Window.FindName('OutputLog')
$Window.FindName('CatalogAddress').Text = $CatalogUrl
$script:ActionButtons = @('StartButton','StopButton','RestartButton','RefreshButton','LogsButton','CatalogButton')

function Set-LauncherStatus {
    param([string] $Text, [string] $BrushName)
    $script:StatusLabel.Text = $Text
    $script:StatusLabel.Foreground = [System.Windows.Media.BrushConverter]::new().ConvertFromString($BrushName)
}
function Invoke-DockerCommand {
    param([Parameter(Mandatory)][string[]] $Arguments)
    if (-not (Get-Command docker -ErrorAction SilentlyContinue)) { throw 'Docker Desktop bulunamadı.' }
    $Result = (& docker @Arguments 2>&1 | Out-String).Trim()
    if ($LASTEXITCODE -ne 0) { throw "DOCKER_COMMAND_FAILED: $Result" }
    return $Result
}
function Update-ServerStatus {
    if (-not (Get-Command docker -ErrorAction SilentlyContinue)) {
        Set-LauncherStatus 'Docker kurulu değil' '#FBBF24'
        return
    }
    try {
        $Running = Invoke-DockerCommand -Arguments @('compose','-f',$ComposeFile,'ps','--status','running','--services')
        if (($Running -split '\r?\n') -contains 'suwayomi') {
            Set-LauncherStatus 'Suwayomi çalışıyor' '#34D399'
        } else {
            Set-LauncherStatus 'Suwayomi durdurulmuş' '#FBBF24'
        }
    } catch {
        Set-LauncherStatus 'Docker bağlantısı kontrol edilmeli' '#F87171'
        $script:OutputLog.Text = "Durum kontrolü: $($_.Exception.Message)"
    }
}
function Invoke-LauncherAction {
    param([string] $Title, [scriptblock] $Operation)
    foreach ($Name in $script:ActionButtons) { $Window.FindName($Name).IsEnabled = $false }
    $script:OutputLog.Text = $Title + [Environment]::NewLine
    try {
        $Result = & $Operation
        if ($null -ne $Result) { $script:OutputLog.AppendText(($Result | Out-String)) }
        $script:OutputLog.AppendText([Environment]::NewLine + 'İşlem tamamlandı.')
    } catch {
        $script:OutputLog.AppendText([Environment]::NewLine + "HATA: $($_.Exception.Message)")
    } finally {
        foreach ($Name in $script:ActionButtons) { $Window.FindName($Name).IsEnabled = $true }
        Update-ServerStatus
        $script:OutputLog.ScrollToEnd()
    }
}
$Window.FindName('StartButton').Add_Click({
    Invoke-LauncherAction 'Suwayomi başlatılıyor...' { & $StartScript 2>&1 | Out-String }
})
$Window.FindName('StopButton').Add_Click({
    Invoke-LauncherAction 'Sunucu durduruluyor (veriler korunur)...' {
        Invoke-DockerCommand -Arguments @('compose','-f',$ComposeFile,'stop','suwayomi')
    }
})
$Window.FindName('RestartButton').Add_Click({
    Invoke-LauncherAction 'Sunucu yeniden başlatılıyor...' {
        Invoke-DockerCommand -Arguments @('compose','-f',$ComposeFile,'restart','suwayomi')
    }
})
$Window.FindName('RefreshButton').Add_Click({ Update-ServerStatus })
$Window.FindName('LogsButton').Add_Click({
    Invoke-LauncherAction 'Son 100 günlük satırı:' {
        Invoke-DockerCommand -Arguments @('compose','-f',$ComposeFile,'logs','--tail','100','--no-color','suwayomi')
    }
})
$Window.FindName('CatalogButton').Add_Click({
    Invoke-LauncherAction 'Katalog bağlantısı kontrol ediliyor...' {
        $Response = Invoke-WebRequest -Uri $CatalogUrl -Method Head -UseBasicParsing -TimeoutSec 15
        "Katalog HTTP durumu: $([int]$Response.StatusCode). Uzantı çalışması ayrıca test edilmelidir."
    }
})
$Window.FindName('CopyButton').Add_Click({
    [System.Windows.Clipboard]::SetText($CatalogUrl)
    $script:OutputLog.Text = 'Katalog adresi panoya kopyalandı. WebUI içindeki uzantı depolarına ekleyin.'
})
$Window.FindName('WebButton').Add_Click({
    try { [void][System.Diagnostics.Process]::Start($WebUrl) }
    catch { $script:OutputLog.Text = "Tarayıcı açılamadı: $($_.Exception.Message)" }
})
$Window.Add_Loaded({ Update-ServerStatus })
[void]$Window.ShowDialog()
