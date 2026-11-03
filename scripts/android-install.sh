#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
ADB="${ADB:-adb}"
serial="${1:-}"
if [[ -z "$serial" ]]; then
  mapfile -t devices < <("$ADB" devices | awk 'NR>1 && $2=="device" {print $1}')
  if [[ "${#devices[@]}" != 1 ]]; then
    echo "Conecte e autorize um único celular ou informe o serial: scripts/android-install.sh SERIAL" >&2
    exit 1
  fi
  serial="${devices[0]}"
fi
apk="apps/mobile/composeApp/build/outputs/apk/debug/composeApp-debug.apk"
[[ -f "$apk" ]] || { echo "Compile o APK primeiro: cd apps/mobile && ./gradlew assembleDebug" >&2; exit 1; }
"$ADB" -s "$serial" reverse tcp:8080 tcp:8080
"$ADB" -s "$serial" install -r "$apk"
"$ADB" -s "$serial" shell am start -n br.com.sorriprecos/.MainActivity
echo "SorriPreços iniciado. Mantenha o backend na porta 8080 e o USB conectado."
