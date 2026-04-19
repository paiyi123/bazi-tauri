# Android Release

## Build Targets

From `E:\Data\src\JavaPrj\bazi`:

```powershell
npm.cmd run tauri:android:build
```

This produces both:

- a signed `arm64` release APK for device installs
- a signed `arm64` release AAB for Play Console upload

Single-target commands:

```powershell
npm.cmd run tauri:android:apk
npm.cmd run tauri:android:aab
```

## Signing

The release pipeline looks for `src-tauri/gen/android/key.properties`.

If it does not exist, the PowerShell build helper generates:

- `src-tauri/gen/android/key.properties`
- `src-tauri/gen/android/keystore/bazi-release.jks`

Back up those two files together. Future Play Store updates must use the same keystore.

If you want to replace the generated keystore, copy `src-tauri/gen/android/key.properties.example` to `key.properties` and point it to your own `.jks` file.

## Version Code Strategy

`versionName` comes from `package.json`.

`versionCode` is generated as:

```text
major * 100000000 + minor * 1000000 + patch * 10000 + buildCounter
```

Default `buildCounter`:

```text
(currentYear - 2020) * 400 + dayOfYear
```

Override it when needed:

```powershell
$env:ANDROID_BUILD_COUNTER = "2601"
npm.cmd run tauri:android:build
```

Override the visible version name if needed:

```powershell
$env:BAZI_ANDROID_VERSION_NAME = "1.0.1-rc1"
```

## Output Paths

APK:

`E:\Data\src\JavaPrj\bazi\src-tauri\gen\android\app\build\outputs\apk\arm64\release`

AAB:

`E:\Data\src\JavaPrj\bazi\src-tauri\gen\android\app\build\outputs\bundle\arm64Release`

## Play Store Notes

- Upload the `.aab`, not the `.apk`.
- Keep `key.properties` and `.jks` out of version control.
- When publishing an update, keep the same `applicationId` and keystore.
