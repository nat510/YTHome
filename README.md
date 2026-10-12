# YTHome

**Turn an old Android phone or tablet into a YouTube player again.**

## Why this exists

Newer versions of the official YouTube app no longer install on older Android versions, and
the old versions that are already installed eventually stop working. Devices with perfectly
good screens, speakers and Wi-Fi get pushed into a drawer or onto the e-waste pile, just
because one app dropped them. Plenty of companies talk about ESG and sustainability; quietly
abandoning working hardware is the opposite.

YTHome keeps those devices useful. It is a tiny launcher that opens YouTube's mobile website
the moment you tap its icon, using the browser that the device still supports.

## What it is (and isn't)

YTHome is a **simple web-page frame**. Tapping the icon opens **m.youtube.com** in a
**Chrome Custom Tab**:

- **Your normal YouTube account**: a Custom Tab shares Chrome's sign-in, so your account,
  recommendations, subscriptions, and YouTube Premium (if you have it) work exactly as on
  m.youtube.com in Chrome.
- **More room for video**: the toolbar slides away while you scroll down, the share button
  and page-title line are turned off, and the toolbar is dark to match YouTube.
- **Nothing in between**: everything you see is YouTube's own mobile website running in
  Chrome. The app does not download, modify or block anything, and it does not collect any
  data. It exits right after opening the page.

It is **not** a YouTube client, downloader or ad blocker, and it does not use any unofficial
YouTube API.

## Requirements

- Android **6.0 (API 23)** or newer, 32-bit or 64-bit ARM (`armeabi-v7a`, `arm64-v8a`).
- **Google Chrome** installed. Without Chrome the page opens in the default browser instead.
- Sign in to YouTube once in Chrome; YTHome uses that session.

## Limits

These come from how Android and Chrome work, not from this app:

- The status (notification) bar and Chrome's toolbar cannot be removed completely; they are
  Chrome's window. Video fullscreen hides both.
- Chrome's close, minimize, site-settings and menu buttons, and the toolbar height, cannot be
  changed by apps.
- Performance is that of m.youtube.com in Chrome on your device. YTHome does not make the
  website lighter; it just gets you there in one tap with more screen space.
- A toolbar-free "app mode" (Trusted Web Activity) would require youtube.com to verify the
  app, which it does not.

## How it works

The APK contains **no Java code**. `AndroidManifest.xml` declares `android:hasCode="false"`
and uses Android's built-in `android.app.NativeActivity`, which loads `libythome.so` (Rust,
`rust/src/lib.rs`). Its `ANativeActivity_onCreate` uses JNI to:

1. build `Intent(ACTION_VIEW, "https://m.youtube.com/")` with the standard Custom Tabs
   extras (session, toolbar hiding on scroll, toolbar colour, share off, no title);
2. start it with package `com.android.chrome`, falling back to the default browser;
3. finish itself.

## Build

Requirements:

- Rust (stable) with targets `armv7-linux-androideabi` and `aarch64-linux-android`
- Android SDK with `platforms;android-35` and `build-tools;35.0.0`
- Android NDK r27c (`ndk;27.2.12479018`)
- JDK 17

```bash
rustup target add armv7-linux-androideabi aarch64-linux-android
export ANDROID_HOME=/path/to/android-sdk
export JAVA_HOME=/path/to/jdk-17
scripts/build.sh all        # or: debug | release
```

APKs are copied to `outputs/`. The release APK is signed only if you create
`android/keystore.properties` (git-ignored):

```
storeFile=/path/to/your-release.jks
storePassword=...
keyAlias=...
keyPassword=...
```

Without it, the build produces `YTHome-<version>-release-unsigned.apk` and a debug APK
signed with your local Android debug key.

## Disclaimer

YTHome is an independent project, not affiliated with or endorsed by Google or YouTube.
YouTube is a trademark of Google LLC. All content shown is YouTube's own website, subject
to YouTube's Terms of Service.
