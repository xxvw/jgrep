# jgrep

`jgrep` स्थानीय रूप से चलने वाला, अर्थ-आधारित खोज के लिए grep-जैसा कमांड है। यह प्राकृतिक-भाषा संदर्भ से इनपुट की संबंधित पंक्तियाँ चुनता है और `grep` वाली फ़ाइल तथा पाइप कार्यप्रणाली बनाए रखता है।

> **v0.1.1:** संस्करण-युक्त नेटिव आर्काइव [GitHub Releases](https://github.com/xxvw/jgrep/releases) पर उपलब्ध हैं। यह परियोजना बेंचमार्क, सटीकता, थ्रूपुट या विलंबता की कोई गारंटी नहीं देती।

डिफ़ॉल्ट अर्थ-आधारित मोड स्थानीय [Qwen2.5-0.5B-Instruct](https://huggingface.co/Qwen/Qwen2.5-0.5B-Instruct) मॉडल से यह द्विआधारी निर्णय लेता है कि प्रत्येक पंक्ति संदर्भ से संबंधित है या नहीं। खोजा गया टेक्स्ट किसी होस्टेड मॉडल को नहीं भेजा जाता; Python, Ollama या लगातार चलने वाली सेवा की जरूरत नहीं है। `-E` Rust रेगुलर एक्सप्रेशन और `-F` निश्चित-स्ट्रिंग मोड चुनते हैं; इनमें मॉडल डाउनलोड या लोड नहीं होता।

यह परियोजना स्वतंत्र रूप से विकसित है और Jev, TypeSafe, Qwen, Hugging Face या llama.cpp से संबद्ध, समर्थित या वितरित नहीं है। Jev केवल द्विआधारी-निर्णय इंटरैक्शन मॉडल की प्रेरणा है।

## इंस्टॉल करें

प्राथमिक इंस्टॉलेशन पथ में `git clone` की जरूरत नहीं है। नीचे दिया गया एक-पेस्ट कमांड `v0.1.1` का संस्करण-पिन किया हुआ इंस्टॉलर बंडल डाउनलोड करता है, उसे निकालने से पहले SHA-256 से जांचता है, और फिर स्थानीय फ़ाइल चलाता है। इसमें `curl | sh` या `Invoke-Expression` का उपयोग नहीं होता। इंस्टॉलर उसके बाद macOS (Apple Silicon या Intel), Windows x64, या Linux x64 (glibc 2.35 या बाद का) के सही नेटिव आर्काइव को डाउनलोड और सत्यापित करता है।

### macOS और Linux

Bash या zsh में यह एक संयुक्त कमांड पेस्ट करें:

```sh
(
  set -e
  version=v0.1.1
  archive="localjev-grep-installers-${version}.tar.gz"
  workdir="$(mktemp -d)"
  trap 'rm -rf "$workdir"' EXIT
  base="https://github.com/xxvw/jgrep/releases/download/${version}"
  curl --fail --silent --show-error --location --proto '=https' \
    --proto-redir '=https' -o "$workdir/$archive" "$base/$archive"
  curl --fail --silent --show-error --location --proto '=https' \
    --proto-redir '=https' -o "$workdir/$archive.sha256" "$base/$archive.sha256"
  (cd "$workdir" && if command -v shasum >/dev/null 2>&1; then
    shasum -a 256 -c "$archive.sha256"
  else
    sha256sum -c "$archive.sha256"
  fi)
  tar -xzf "$workdir/$archive" -C "$workdir"
  bash "$workdir/localjev-grep-installers-${version}/installers/hi/install.sh" \
    --version "$version"
)
```

### Windows PowerShell

PowerShell में यह एक कमांड ब्लॉक पेस्ट करें:

```powershell
& {
  $ErrorActionPreference = 'Stop'
  $version = 'v0.1.1'
  $archive = "localjev-grep-installers-$version.zip"
  $workdir = Join-Path ([System.IO.Path]::GetTempPath()) ([System.Guid]::NewGuid())
  New-Item -ItemType Directory -Path $workdir | Out-Null
  try {
    $base = "https://github.com/xxvw/jgrep/releases/download/$version"
    Invoke-WebRequest -UseBasicParsing -Uri "$base/$archive" -OutFile (Join-Path $workdir $archive)
    Invoke-WebRequest -UseBasicParsing -Uri "$base/$archive.sha256" -OutFile (Join-Path $workdir "$archive.sha256")
    $manifest = (Get-Content -LiteralPath (Join-Path $workdir "$archive.sha256") -Raw).Trim()
    $manifestPattern = '^[A-Fa-f0-9]{64}  ' + [regex]::Escape($archive) + '$'
    if ($manifest -notmatch $manifestPattern) { throw 'installer bundle checksum manifest is invalid' }
    $expected = $manifest.Substring(0, 64).ToLowerInvariant()
    $actual = (Get-FileHash -LiteralPath (Join-Path $workdir $archive) -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($actual -ne $expected) { throw 'installer bundle checksum mismatch' }
    Expand-Archive -LiteralPath (Join-Path $workdir $archive) -DestinationPath $workdir -Force
    & (Join-Path $workdir "localjev-grep-installers-$version\installers\hi\install.ps1") -Version $version
  } finally {
    Remove-Item -LiteralPath $workdir -Recurse -Force -ErrorAction SilentlyContinue
  }
}
```

इस पृष्ठ में हिंदी रैपर (`hi`) चुना गया है। हर भाषा रैपर केवल स्थानीय भाषा का आरंभ संदेश दिखाता है और साझा सत्यापित कोर इंस्टॉलर को ही चलाता है; वह हर विकल्प को उसी तक भेजता है। अनुकूलन के लिए macOS/Linux वाले ब्लॉक में अंतिम `bash` इंस्टॉलर कॉल पर, `--version "$version"` के बाद, `--install-dir ~/bin` जोड़ें—उसे ब्लॉक के बाद न जोड़ें। इसी तरह PowerShell ब्लॉक के अंतिम इंस्टॉलर कॉल में `-InstallDir C:\bin` जोड़ें।

macOS/Linux पर, यदि `XDG_BIN_HOME` सेट नहीं है, तो डिफ़ॉल्ट इंस्टॉल स्थान `$HOME/.local/bin` है। यदि वह पहले से `PATH` में नहीं है, तो अपने शेल कॉन्फ़िगरेशन में यह जोड़ें:

```sh
export PATH="$HOME/.local/bin:$PATH"
```

PowerShell ब्लॉक में इंस्टॉलर कॉल की अंतिम पंक्ति में `-AddToPath` जोड़ें (`... -Version $version -AddToPath`), ताकि इंस्टॉल निर्देशिका वर्तमान उपयोगकर्ता के `PATH` और आगे की सत्रों में जोड़ी जाए।

### स्रोत से बनाना (वैकल्पिक)

स्रोत से बनाने के लिए `rust-toolchain.toml` में पिन किया Rust टूलचेन, CMake और एम्बेड किए गए llama.cpp को बनाने वाला C++ कंपाइलर चाहिए। यदि आप स्वयं बिल्ड करना चाहते हैं, असमर्थित आर्किटेक्चर पर काम कर रहे हैं, या पुराने Linux पर हैं जो glibc 2.35 की शर्त पूरी नहीं करता, तो इस विकल्प का उपयोग करें:

```sh
git clone https://github.com/xxvw/jgrep.git
cd jgrep
cargo build --release
./target/release/jgrep --help
```

Windows PowerShell में:

```powershell
git clone https://github.com/xxvw/jgrep.git
Set-Location jgrep
cargo build --release
.\target\release\jgrep.exe --help
```

हर समर्थित प्लेटफ़ॉर्म पर CPU उपलब्ध है; Apple Silicon पर स्वचालित डिवाइस सेटिंग Metal का उपयोग कर सकती है। कोडिंग एजेंट में संक्षिप्त `--ai` खोज सेट करने के लिए [हिंदी एजेंट-प्लगइन मार्गदर्शिका](plugins/jgrep-agent/README.hi.md) देखें।

## उपयोग

सामान्य आर्ग्युमेंट क्रम:

```text
jgrep [OPTIONS] <context> [FILE ...]
```

फ़ाइल आर्ग्युमेंट न देने पर, या फ़ाइल के स्थान पर `-` देने पर, `jgrep` मानक इनपुट पढ़ता है। स्थितिगत `<context>` के बदले `-e` दिया जा सकता है; कई `-e` संदर्भ OR किए जाते हैं।

### Bash

```sh
# डिफ़ॉल्ट मॉडल पहले डाउनलोड करें; बिना संदर्भ के डाउनलोड के बाद कमांड समाप्त होती है
jgrep --download-model

# डिफ़ॉल्ट स्थानीय अर्थ-आधारित खोज
jgrep 'डेटाबेस लॉगिन अस्वीकार हुआ' service.log

# पाइप, रिकर्सिव खोज और आसपास की पंक्तियाँ
journalctl -f | jgrep --line-buffered 'कनेक्शन रीसेट हुआ'
jgrep -n -r --include '*.rs' 'फ़ाइल सिस्टम त्रुटि संभालना' src/
jgrep -C 2 'अनुरोध का समय समाप्त हुआ' app.log

# नेटवर्क का उपयोग नहीं करता; मान्य स्थानीय मॉडल न हो तो असफल होता है
jgrep --offline 'प्रमाणीकरण विफल हुआ' service.log

# सामान्य पाठ खोज; मॉडल शुरू नहीं होता
jgrep -E -i 'error|warning' app.log
jgrep -F 'connection refused' app.log
```

### PowerShell

```powershell
# रिलीज़ आर्काइव निकालने के बाद उसकी निर्देशिका में चलाएँ
.\jgrep.exe --download-model
Get-Content .\service.log | .\jgrep.exe 'डेटाबेस लॉगिन अस्वीकार हुआ'
.\jgrep.exe --offline 'प्रमाणीकरण विफल हुआ' .\service.log
.\jgrep.exe -E -i 'error|warning' .\app.log
```

## प्रमुख विकल्प

| विकल्प | काम |
| --- | --- |
| `-e <context>` | एक संदर्भ जोड़ता है; कोई भी संदर्भ मिलने पर पंक्ति चुनी जाती है। |
| `-E` / `-F` | Rust रेगुलर एक्सप्रेशन / निश्चित-स्ट्रिंग मोड; दोनों साथ नहीं चल सकते। |
| `-i` | अक्षर-आकार की अनदेखी; केवल `-E` और `-F` में। |
| `-n`, `-H` / `-h`, `-c` | पंक्ति संख्या, फ़ाइल नाम हमेशा दिखाना / छिपाना, प्रति इनपुट चुनी पंक्तियों की गिनती। |
| `-l` / `-L`, `-q`, `-m <NUM>`, `-v` | मैच वाली / बिना मैच वाली फ़ाइलों के नाम, पहले चयन पर शांत रूप से रुकना, चयनित पंक्तियों की सीमा, अंतिम चयन उलटना। |
| `-r`, `-A` / `-B` / `-C` | निर्देशिकाओं में रिकर्सिव खोज और बाद / पहले / आस-पास की पंक्तियाँ। |
| `--include <GLOB>` / `--exclude <GLOB>` | रिकर्सिव खोज में पथ शामिल या छोड़ें। |
| `--color <auto\|always\|never>`, `--line-buffered` | ANSI हाइलाइटिंग नियंत्रित करें; स्ट्रीमिंग पाइप के लिए हर आउटपुट पंक्ति फ्लश करें। |
| `--threshold <0..1>`, `--score` | अर्थ-संबंधितता सीमा (डिफ़ॉल्ट `0.5`) और आउटपुट में स्कोर। |
| `--ai`, `--ai-max-results <NUM>` | कोडिंग एजेंटों के लिए संक्षिप्त स्थान आउटपुट और उसकी सीमा। |
| `--model <PATH>`, `--download-model`, `--offline`, `--device <auto\|cpu>` | स्थानीय GGUF मॉडल, डिफ़ॉल्ट मॉडल डाउनलोड, नेटवर्क निषेध और स्थानीय अनुमान डिवाइस। |

अर्थ-आधारित विकल्प लेक्सिकल मोड में मान्य नहीं हैं और `--offline` को `--download-model` के साथ नहीं दिया जा सकता। `-` से शुरू होने वाले पैटर्न या पथ के पहले `--` प्रयोग करें।

`--ai` केवल संक्षिप्त `path:line` स्थान देता है: मिलान वाले स्रोत-पाठ, ANSI
रंग, स्कोर या संदर्भ पंक्तियों के बिना। पूरे एक रन में डिफ़ॉल्ट अधिकतम 50
स्थान हैं; `--ai-max-results` से सीमा बदलें। इसके बाद कोडिंग एजेंट केवल
ज़रूरी संकीर्ण पंक्ति-सीमाएँ ले सकता है, जिससे टूल कॉलिंग में टोकन उपयोग
कम होता है।

## मॉडल, गोपनीयता और सीमाएँ

अर्थ-आधारित मोड आधिकारिक Qwen2.5-0.5B-Instruct GGUF **Q8_0** फ़ाइल (लगभग 676 MB) प्रयोग करता है। प्रोग्राम मॉडल संशोधन और SHA-256 को पिन करता है; पहली अर्थ-आधारित खोज या `--download-model` पर उसे डाउनलोड, सत्यापित और उपयोगकर्ता के ऐप कैश में परमाणु रूप से रखता है, फिर उसे फिर से उपयोग करता है। किसी मौजूदा स्थानीय GGUF फ़ाइल के लिए `--model /path/to/model.gguf` दें।

मॉडल डाउनलोड करने के बाद खोज पूरी तरह ऑफ़लाइन चल सकती है। `--help`, लेक्सिकल खोज, खाली इनपुट और `-m 0` मॉडल आरंभ नहीं करते। अर्थ स्कोर मॉडल के अगले `Yes` और `No` टोकन के logits के अंतर से बनता है:

```text
sigmoid(logit(Yes) - logit(No))
```

यह स्कोर संबंधितता स्कोर है, कैलिब्रेट की गई प्रायिकता या शुद्धता की गारंटी नहीं। अस्पष्ट भाषा, नकार, भाषा-भेद, लंबी पंक्तियाँ और खोजी फ़ाइलों की विरोधी सामग्री परिणाम बदल सकती है। अर्थ-प्रॉम्प्ट की सीमा 4,096 टोकन है; बहुत लंबी पंक्ति चुपचाप नहीं काटी जाती। जीवन-सुरक्षा, कानूनी, चिकित्सा या सूचना-सुरक्षा संबंधी महत्वपूर्ण निर्णयों के लिए इसे अकेला आधार न बनाएं। परियोजना `grep` जितनी गति या मापी हुई सटीकता, थ्रूपुट अथवा विलंबता की गारंटी नहीं देती।

`jgrep` इनपुट को क्रमिक रूप से पढ़ता है, चयनित पंक्तियाँ इनपुट क्रम में देता है और UTF-8, LF / CRLF तथा Unicode पथ समर्थित हैं। रिकर्सिव खोज निर्देशिका symlink का अनुसरण नहीं करती; उसमें मिले बाइनरी फ़ाइलों को निदान के साथ छोड़ दिया जाता है। `grep` की तरह निकास स्थिति: चयन मिलने पर `0`, न मिलने पर `1`, और त्रुटि पर `2`।

## लाइसेंस

परियोजना स्रोत कोड **GPL-3.0-or-later** के अंतर्गत लाइसेंस प्राप्त है। किसी बिल्ड को पुनर्वितरित करने से पहले [LICENSE](LICENSE), [NOTICE](NOTICE), और [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md) पढ़ें। डिफ़ॉल्ट Qwen मॉडल अलग से डाउनलोड होने वाली Apache-2.0 संपत्ति है और उसे परियोजना स्रोत कोड की शर्तों में लाइसेंस नहीं किया जाता।
