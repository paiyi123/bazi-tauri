param(
  [Parameter(Mandatory = $true, Position = 0)]
  [string]$Subcommand,

  [Parameter(ValueFromRemainingArguments = $true)]
  [string[]]$Args
)

function Resolve-FirstExistingPath {
  param([string[]]$Candidates)

  foreach ($candidate in $Candidates) {
    if ($candidate -and (Test-Path -LiteralPath $candidate)) {
      return (Resolve-Path -LiteralPath $candidate).Path
    }
  }

  return $null
}

function Resolve-LatestChildDirectory {
  param([string]$Parent)

  if (!(Test-Path -LiteralPath $Parent)) {
    return $null
  }

  return Get-ChildItem -LiteralPath $Parent -Directory |
    Sort-Object Name -Descending |
    Select-Object -ExpandProperty FullName -First 1
}

function New-RandomSecret {
  param([int]$Length = 24)

  $alphabet = "abcdefghijkmnopqrstuvwxyzABCDEFGHJKLMNPQRSTUVWXYZ23456789!@$%*+-_=.:"
  $bytes = New-Object byte[] $Length
  [System.Security.Cryptography.RandomNumberGenerator]::Create().GetBytes($bytes)

  $chars = for ($index = 0; $index -lt $Length; $index++) {
    $alphabet[$bytes[$index] % $alphabet.Length]
  }

  return -join $chars
}

function Get-PackageVersion {
  param([string]$PackageJsonPath)

  $package = Get-Content -LiteralPath $PackageJsonPath -Raw | ConvertFrom-Json
  return [string]$package.version
}

function Get-AndroidBuildCounter {
  if ($env:ANDROID_BUILD_COUNTER) {
    $override = [int]$env:ANDROID_BUILD_COUNTER
    if ($override -lt 0 -or $override -gt 9999) {
      throw "ANDROID_BUILD_COUNTER must be between 0 and 9999."
    }

    return $override
  }

  $now = Get-Date
  $counter = (($now.Year - 2020) * 400) + $now.DayOfYear
  if ($counter -lt 0 -or $counter -gt 9999) {
    throw "Computed build counter $counter is out of range. Set ANDROID_BUILD_COUNTER explicitly."
  }

  return $counter
}

function Get-AndroidVersionCode {
  param(
    [string]$Version,
    [int]$BuildCounter
  )

  if ($Version -notmatch '^(\d+)\.(\d+)\.(\d+)') {
    throw "Version '$Version' must start with semantic version segments like 1.0.0."
  }

  $major = [int]$Matches[1]
  $minor = [int]$Matches[2]
  $patch = [int]$Matches[3]

  if ($minor -gt 99 -or $patch -gt 99) {
    throw "Minor and patch versions must stay below 100 for Android versionCode generation."
  }

  $versionCode = ($major * 100000000) + ($minor * 1000000) + ($patch * 10000) + $BuildCounter
  if ($versionCode -gt [int]::MaxValue) {
    throw "Android versionCode $versionCode exceeds the Int32 limit. Bump the scheme or lower the major version."
  }

  return $versionCode
}

function Ensure-AndroidSigningConfig {
  param(
    [string]$AndroidProject,
    [string]$KeytoolCommand
  )

  $keyPropertiesPath = Join-Path $AndroidProject "key.properties"
  if (Test-Path -LiteralPath $keyPropertiesPath) {
    $properties = @{}
    foreach ($line in Get-Content -LiteralPath $keyPropertiesPath) {
      if ($line -match '^\s*([^#=]+?)=(.*)$') {
        $properties[$Matches[1].Trim()] = $Matches[2].Trim()
      }
    }

    if (
      $properties["storeFile"] -eq "keystore/bazi-release.jks" -and
      $properties["storePassword"] -and
      $properties["keyAlias"] -and
      $properties["keyPassword"] -ne $properties["storePassword"]
    ) {
      @"
storeFile=$($properties["storeFile"])
storePassword=$($properties["storePassword"])
keyAlias=$($properties["keyAlias"])
keyPassword=$($properties["storePassword"])
"@ | Set-Content -LiteralPath $keyPropertiesPath -NoNewline
    }

    return
  }

  $keystoreDir = Join-Path $AndroidProject "keystore"
  $keystorePath = Join-Path $keystoreDir "bazi-release.jks"
  $storePassword = New-RandomSecret
  $keyPassword = $storePassword
  $keyAlias = "bazi-release"

  New-Item -ItemType Directory -Force -Path $keystoreDir | Out-Null

  & $KeytoolCommand -genkeypair `
    -keystore $keystorePath `
    -storepass $storePassword `
    -keypass $keyPassword `
    -alias $keyAlias `
    -keyalg RSA `
    -keysize 4096 `
    -sigalg SHA256withRSA `
    -validity 9125 `
    -dname "CN=BaZi Release,O=BaZi,C=TW"

  if ($LASTEXITCODE -ne 0) {
    throw "Failed to generate Android release keystore."
  }

  @"
storeFile=keystore/bazi-release.jks
storePassword=$storePassword
keyAlias=$keyAlias
keyPassword=$keyPassword
"@ | Set-Content -LiteralPath $keyPropertiesPath -NoNewline
}

function Invoke-AndroidAssetGeneration {
  param(
    [string]$PythonCommand,
    [string]$WorkspaceRoot
  )

  $assetScript = Join-Path $WorkspaceRoot "scripts\generate-android-assets.py"
  if (!(Test-Path -LiteralPath $assetScript)) {
    throw "Android asset generator not found at $assetScript"
  }

  $pythonLeaf = Split-Path -Leaf $PythonCommand
  if ($pythonLeaf -ieq "py.exe") {
    & $PythonCommand -3 $assetScript
  } else {
    & $PythonCommand $assetScript
  }

  if ($LASTEXITCODE -ne 0) {
    throw "Android asset generation failed."
  }
}

function Invoke-AndroidWebViewWorkaround {
  param([string]$WorkspaceRoot)

  $rustWebViewPath = Join-Path $WorkspaceRoot "src-tauri\gen\android\app\src\main\java\com\bazi\desktop\generated\RustWebView.kt"
  $ipcPath = Join-Path $WorkspaceRoot "src-tauri\gen\android\app\src\main\java\com\bazi\desktop\generated\Ipc.kt"
  if (!(Test-Path -LiteralPath $rustWebViewPath) -or !(Test-Path -LiteralPath $ipcPath)) {
    return
  }

  $content = Get-Content -LiteralPath $rustWebViewPath -Raw

  if ($content.Contains("import android.util.Base64")) {
    $content = $content.Replace(
@"
import android.annotation.SuppressLint
import android.util.Base64
import android.webkit.*
"@,
@"
import android.annotation.SuppressLint
import android.webkit.*
"@
    )
  }

  $content = $content.Replace(
@"
    override fun loadUrl(url: String) {
        if (!shouldOverride(url)) {
            super.loadUrl(url);
        }
    }
"@,
@"
    override fun loadUrl(url: String) {
        if (!shouldOverride(url)) {
            if (!loadBundledAppPage(url)) {
                super.loadUrl(url)
            }
        }
    }
"@
  )

  $content = $content.Replace(
@"
    override fun loadUrl(url: String, additionalHttpHeaders: Map<String, String>) {
        if (!shouldOverride(url)) {
            super.loadUrl(url, additionalHttpHeaders);
        }
    }
"@,
@"
    override fun loadUrl(url: String, additionalHttpHeaders: Map<String, String>) {
        if (!shouldOverride(url)) {
            if (!loadBundledAppPage(url)) {
                super.loadUrl(url, additionalHttpHeaders)
            }
        }
    }
"@
  )

  if (-not $content.Contains("private fun loadBundledAppPage")) {
    $content = $content.Replace(
@"
    private external fun shouldOverride(url: String): Boolean
"@,
@"
    private fun loadBundledAppPage(url: String): Boolean {
        if (!isBundledAppUrl(url)) {
            return false
        }

        return try {
            val html = context.assets.open("index.inline.html").bufferedReader(Charsets.UTF_8).use { it.readText() }
            val injectedScripts = initScripts.joinToString(separator = "\n") { "<script>\n$it\n</script>" }
            val pageHtml = if (html.contains("<head>")) {
                html.replaceFirst("<head>", "<head>\n$injectedScripts\n")
            } else {
                "$injectedScripts\n$html"
            }
            super.loadDataWithBaseURL(url, pageHtml, "text/html", "utf-8", url)
            true
        } catch (_: Exception) {
            false
        }
    }

    private fun isBundledAppUrl(url: String): Boolean {
        return url.startsWith("https://tauri.localhost") || url.startsWith("http://tauri.localhost")
    }

    private external fun shouldOverride(url: String): Boolean
"@
    )
  }

  Set-Content -LiteralPath $rustWebViewPath -Value $content -NoNewline

  $ipcContent = Get-Content -LiteralPath $ipcPath -Raw
  $ipcContent = $ipcContent.Replace(
@"
            // we're not using WebView::getUrl() here because it needs to be executed on the main thread
            // and it would slow down the Ipc
            // so instead we track the current URL on the webview client
            this.ipc(webViewClient.currentUrl, m)
"@,
@"
            // we're not using WebView::getUrl() here because it needs to be executed on the main thread
            // and it would slow down the Ipc
            // so instead we track the current URL on the webview client.
            // When the page is loaded with loadData* the tracked URL may become a data: URI,
            // which is too long for Wry's Android IPC request builder.
            val currentUrl = webViewClient.currentUrl
            val ipcUrl = if (currentUrl.startsWith("data:") || currentUrl == "about:blank") {
                "https://tauri.localhost/"
            } else {
                currentUrl
            }
            this.ipc(ipcUrl, m)
"@
  )

  Set-Content -LiteralPath $ipcPath -Value $ipcContent -NoNewline
}

$workspaceRoot = Split-Path -Parent $PSScriptRoot
$npmCommand = Resolve-FirstExistingPath @(
  $env:TAURI_NPM_COMMAND,
  $env:NPM_COMMAND,
  "D:\soft\nodejs\npm.cmd",
  "D:\Soft\nodejs\npm.cmd",
  "C:\Program Files\nodejs\npm.cmd"
)

if (-not $npmCommand) {
  throw "Unable to locate npm.cmd. Install Node.js or set TAURI_NPM_COMMAND."
}

$nodeDir = Split-Path -Parent $npmCommand
$cargoCommand = Resolve-FirstExistingPath @(
  (Join-Path $env:USERPROFILE ".cargo\bin\cargo.exe"),
  (Join-Path $env:USERPROFILE ".cargo\bin\cargo")
)

if (-not $cargoCommand) {
  throw "Unable to locate cargo. Install Rust or add cargo to your profile."
}

$cargoDir = Split-Path -Parent $cargoCommand
$sdkRoot = Resolve-FirstExistingPath @(
  $env:ANDROID_HOME,
  $env:ANDROID_SDK_ROOT,
  "D:\Soft\Android\SDK",
  "${env:LOCALAPPDATA}\Android\Sdk"
)

if (-not $sdkRoot) {
  throw "Unable to locate Android SDK. Set ANDROID_HOME or ANDROID_SDK_ROOT."
}

$javaHome = Resolve-FirstExistingPath @(
  $env:JAVA_HOME,
  "D:\Soft\jdk\jdk-17.0.14+7",
  "C:\Program Files\Android\Android Studio\jbr",
  "C:\Program Files\Java\jdk-17"
)

if (-not $javaHome) {
  throw "Unable to locate JDK. Set JAVA_HOME."
}

$keytoolCommand = Resolve-FirstExistingPath @(
  (Join-Path $javaHome "bin\keytool.exe")
)

if (-not $keytoolCommand) {
  throw "Unable to locate keytool in JAVA_HOME."
}

$pythonCommand = Resolve-FirstExistingPath @(
  "C:\Users\paiyi123\AppData\Local\Programs\Python\Launcher\py.exe",
  "C:\Users\paiyi123\AppData\Local\Programs\Python\Python313\python.exe",
  "C:\Users\paiyi123\AppData\Local\Microsoft\WindowsApps\python.exe"
)

if (-not $pythonCommand) {
  throw "Unable to locate Python. Install Python 3 or set a usable launcher in PATH."
}

$ndkHome = Resolve-FirstExistingPath @(
  $env:NDK_HOME,
  (Resolve-LatestChildDirectory (Join-Path $sdkRoot "ndk"))
)

if (-not $ndkHome) {
  throw "Unable to locate Android NDK. Install the NDK or set NDK_HOME."
}

$ndkToolchainBin = Resolve-FirstExistingPath @(
  (Join-Path $ndkHome "toolchains\llvm\prebuilt\windows-x86_64\bin")
)

if (-not $ndkToolchainBin) {
  throw "Unable to locate Android LLVM toolchain inside the NDK."
}

$cmdlineToolsBin = Resolve-FirstExistingPath @(
  (Join-Path $sdkRoot "cmdline-tools\latest\bin"),
  (Join-Path $sdkRoot "cmdline-tools\bin")
)

$pathEntries = @(
  $cargoDir,
  (Join-Path $javaHome "bin"),
  (Join-Path $sdkRoot "platform-tools"),
  $cmdlineToolsBin,
  $ndkToolchainBin,
  $nodeDir
) | Where-Object { $_ -and (Test-Path -LiteralPath $_) }

$env:JAVA_HOME = $javaHome
$env:ANDROID_HOME = $sdkRoot
$env:ANDROID_SDK_ROOT = $sdkRoot
$env:NDK_HOME = $ndkHome
$env:TAURI_NPM_COMMAND = $npmCommand
$env:CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER = (Join-Path $ndkToolchainBin "aarch64-linux-android24-clang.cmd")
$env:CC_aarch64_linux_android = $env:CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER
$env:AR_aarch64_linux_android = (Join-Path $ndkToolchainBin "llvm-ar.exe")
$env:Path = (($pathEntries + $env:Path.Split(';')) | Where-Object { $_ } | Select-Object -Unique) -join ';'

Write-Output "JAVA_HOME=$javaHome"
Write-Output "ANDROID_HOME=$sdkRoot"
Write-Output "NDK_HOME=$ndkHome"
Write-Output "TAURI_NPM_COMMAND=$npmCommand"

if ($Subcommand -eq "build") {
  $androidProject = Join-Path $workspaceRoot "src-tauri\gen\android"
  $gradle = Join-Path $androidProject "gradlew.bat"
  if (!(Test-Path -LiteralPath $gradle)) {
    throw "Android Gradle project not found. Run tauri:android:init first."
  }

  $isDebug = $Args -contains "--debug"
  $isRelease = -not $isDebug
  $buildApk = $Args -contains "--apk"
  $buildAab = $Args -contains "--aab"
  if (-not $buildApk -and -not $buildAab) {
    $buildApk = $true
    $buildAab = $true
  }

  $profileDir = if ($isRelease) { "release" } else { "debug" }
  $cargoArgs = @(
    "build",
    "--manifest-path",
    (Join-Path $workspaceRoot "src-tauri\Cargo.toml"),
    "--target",
    "aarch64-linux-android"
  )
  if ($isRelease) {
    $cargoArgs += "--release"
  }

  if ($isRelease) {
    Ensure-AndroidSigningConfig -AndroidProject $androidProject -KeytoolCommand $keytoolCommand

    $versionName = if ($env:BAZI_ANDROID_VERSION_NAME) {
      $env:BAZI_ANDROID_VERSION_NAME
    } else {
      Get-PackageVersion (Join-Path $workspaceRoot "package.json")
    }
    $buildCounter = Get-AndroidBuildCounter
    $versionCode = Get-AndroidVersionCode -Version $versionName -BuildCounter $buildCounter

    $env:BAZI_ANDROID_VERSION_NAME = $versionName
    $env:BAZI_ANDROID_VERSION_CODE = "$versionCode"

    Write-Output "BAZI_ANDROID_VERSION_NAME=$versionName"
    Write-Output "BAZI_ANDROID_VERSION_CODE=$versionCode"
  }

  & $npmCommand run build
  if ($LASTEXITCODE -ne 0) {
    exit $LASTEXITCODE
  }

  Invoke-AndroidAssetGeneration -PythonCommand $pythonCommand -WorkspaceRoot $workspaceRoot
  Invoke-AndroidWebViewWorkaround -WorkspaceRoot $workspaceRoot

  & $cargoCommand @cargoArgs
  if ($LASTEXITCODE -ne 0) {
    exit $LASTEXITCODE
  }

  $jniDir = Join-Path $androidProject "app\src\main\jniLibs\arm64-v8a"
  $libPath = Join-Path $workspaceRoot "src-tauri\target\aarch64-linux-android\$profileDir\libbazi_lib.so"
  if (!(Test-Path -LiteralPath $libPath)) {
    throw "Built Android library not found at $libPath"
  }

  New-Item -ItemType Directory -Force -Path $jniDir | Out-Null
  Copy-Item -Force $libPath (Join-Path $jniDir "libbazi_lib.so")

  $tasks = [System.Collections.Generic.List[string]]::new()
  $skipTask = if ($isRelease) { "rustBuildArm64Release" } else { "rustBuildArm64Debug" }
  $apkTask = if ($isRelease) { "assembleArm64Release" } else { "assembleArm64Debug" }
  $aabTask = if ($isRelease) { "bundleArm64Release" } else { "bundleArm64Debug" }
  if ($buildApk) {
    $tasks.Add($apkTask)
  }
  if ($buildAab) {
    $tasks.Add($aabTask)
  }
  $tasks.Add("-x")
  $tasks.Add($skipTask)

  Push-Location $androidProject
  try {
    & $gradle @tasks
    exit $LASTEXITCODE
  } finally {
    Pop-Location
  }
}

& $npmCommand run tauri -- android $Subcommand @Args
exit $LASTEXITCODE
