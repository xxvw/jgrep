# jgrep

`jgrep` هو أمر بحث محلي بأسلوب `grep`: يطبع أسطر الإدخال التي يتوافق معناها مع سياق مكتوب بلغة طبيعية، مع الإبقاء على أسلوب العمل المعتاد بالملفات والأنابيب.

> **الإصدار 0.1.1:** تتوفر أرشيفات أصلية مُرقّمة للإصدارات عبر [GitHub Releases](https://github.com/xxvw/jgrep/releases). لا يقدّم المشروع ضمانات للدقة أو نتائج معيارية للأداء.

في الوضع الافتراضي، يستخدم `jgrep` نموذجًا محليًا لاتخاذ قرار صلة ثنائي لكل سطر. لا يرسل النص الذي تبحث فيه إلى نموذج مستضاف، ولا يحتاج إلى Python أو Ollama أو خدمة تعمل في الخلفية. المشروع مستقل، وليس تابعًا لـ Jev أو TypeSafe أو Qwen أو Hugging Face أو llama.cpp، ولا يحظى بتأييد أيٍّ منها، ولا يُعدّ توزيعًا لأيٍّ منها.

## التثبيت

مسار التثبيت الأساسي لا يحتاج إلى `git clone`: ينزّل الأمر التالي حزمة المثبّت الثابتة للإصدار `v0.1.1`، ويتحقق من SHA-256 قبل فكّها، ثم يشغّل ملفًا محليًا. لا يستخدم `curl | sh` أو `Invoke-Expression`. ينزّل المثبّت بعد ذلك أرشيفًا أصليًا مناسبًا لـ macOS (Apple Silicon أو Intel)، أو Windows x64، أو Linux x64 (glibc 2.35 أو أحدث)، ويتحقق منه أيضًا.

يسبق الإصدار `v0.1.1` إعادة تسمية المستودع، لذلك تحتفظ أسماء أرشيفاته ومجلدات مثبّتاته الثابتة بالبادئة `localjev-grep`. تستخدم روابط المستودع والمصدر الحالي والأمر المثبّت الاسم `jgrep`.

### macOS وLinux

الصق هذا الأمر المركب الواحد في Bash أو zsh:

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
  JGREP_REPOSITORY=xxvw/jgrep \
    bash "$workdir/localjev-grep-installers-${version}/installers/ar/install.sh" \
    --version "$version"
)
```

### Windows PowerShell

الصق كتلة الأمر الواحدة التالية في PowerShell:

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
    & (Join-Path $workdir "localjev-grep-installers-$version\installers\ar\install.ps1") -Version $version -Repository 'xxvw/jgrep'
  } finally {
    Remove-Item -LiteralPath $workdir -Recurse -Force -ErrorAction SilentlyContinue
  }
}
```

تستخدم هذه الصفحة الغلاف العربي (`ar`). كل غلاف للغات هو رسالة بدء مترجمة فقط، ويفوّض التثبيت إلى المثبّت الأساسي المشترك المتحقق منه؛ ويمرّر إليه كل الخيارات. للتخصيص، أضف `--install-dir ~/bin` إلى استدعاء `bash` الأخير داخل كتلة macOS أو Linux، بعد `--version "$version"`، ولا تضفه بعد الكتلة. وبالمثل، أضف `-InstallDir C:\bin` إلى استدعاء المثبّت الأخير داخل كتلة PowerShell.

بعد نجاح التثبيت في macOS أو Linux، يكون مسار التثبيت الافتراضي هو `$HOME/.local/bin` عندما لا يعيَّن `XDG_BIN_HOME`. أضفه إلى `PATH` في إعدادات الصدفة إذا لم يكن موجودًا:

```sh
export PATH="$HOME/.local/bin:$PATH"
```

في PowerShell، أضف `-AddToPath` إلى استدعاء المثبّت الأخير داخل كتلة الأمر (`... -Version $version -AddToPath`) لإضافة مجلد التثبيت إلى `PATH` للمستخدم الحالي والجلسات المستقبلية.

### البناء من المصدر (اختياري)

للبناء من المصدر، يتطلب المشروع سلسلة أدوات Rust المحددة في `rust-toolchain.toml`، وCMake، ومترجم C++ لبناء اعتماد llama.cpp المضمّن. استخدم هذا المسار إذا أردت بناء البرنامج بنفسك، أو كنت تعمل على بنية غير مدعومة، أو كان نظام Linux أقدم ولا يفي بمتطلب glibc 2.35:

**Bash / zsh:**

```sh
git clone https://github.com/xxvw/jgrep.git
cd jgrep
cargo build --release
./target/release/jgrep --help
```

**Windows PowerShell:**

```powershell
git clone https://github.com/xxvw/jgrep.git
Set-Location jgrep
cargo build --release
.\target\release\jgrep.exe --help
```

لإعداد وكيل برمجة مثل Codex لاستخدام البحث المختصر `--ai`، راجع [دليل إضافة الوكيل باللغة العربية](plugins/jgrep-agent/README.ar.md).

## الاستخدام

الصيغة المعتادة هي:

```text
jgrep [OPTIONS] <context> [FILE ...]
```

إذا لم تمرر ملفًا، أو مررت `-`، يقرأ `jgrep` الإدخال القياسي. يمكن أن يحل `-e` محل السياق الموضعي، وتُعامل تكراراته كشرط OR.

**Bash / zsh:**

```sh
# البحث الدلالي هو الوضع الافتراضي
jgrep 'a database login was rejected' service.log

# نزّل النموذج الافتراضي مسبقًا؛ من دون سياق ينتهي الأمر بعد التنزيل
jgrep --download-model

# ابحث في تدفق دون تجميعه كاملًا في الذاكرة
journalctl -f | jgrep --line-buffered 'connection was reset'

# بحث متكرر مع أرقام الأسطر
jgrep -n -r --include '*.rs' 'handling a filesystem error' src/

# لا تسمح بأي اتصال شبكي
jgrep --offline 'request timed out' app.log
```

**Windows PowerShell:**

```powershell
.\target\release\jgrep.exe --download-model
Get-Content .\service.log | .\target\release\jgrep.exe 'a database login was rejected'
.\target\release\jgrep.exe --offline 'request timed out' .\app.log
```

للمطابقة التقليدية التي لا تهيئ النموذج ولا تنزّله:

```sh
jgrep -E -i 'error|warning' app.log
jgrep -F 'connection refused' app.log
```

## الأوضاع والخيارات المهمة

| الوضع أو الخيار | الغرض |
| --- | --- |
| الدلالي (افتراضي) | يحكم نموذج محلي على صلة كل سطر بالسياق. |
| `-E` | تعبيرات Rust النمطية؛ لا يدعم GNU BRE أو PCRE أو المراجع الخلفية. |
| `-F` | مطابقة سلسلة نصية حرفية. |
| `-e <context>` | إضافة سياق؛ يكفي تطابق أي سياق لاختيار السطر. |
| `-n` | إضافة رقم السطر إلى الناتج. |
| `-H`، `-h` | إظهار اسم الملف دائمًا، أو إخفاؤه. |
| `-c`، `-l`، `-L`، `-q` | عدّ الأسطر المختارة، أو طباعة أسماء الملفات بحسب وجود تطابق، أو التوقف عند أول اختيار. |
| `-m <NUM>`، `-v` | حدّ عدد الأسطر المختارة لكل إدخال، أو اعكس الاختيار النهائي. لا ينفذ `-m 0` أي عمل نموذجي. |
| `-r`، `-A`، `-B`، `-C` | بحث متكرر، أو أسطر لاحقة أو سابقة أو محيطة. |
| `--include <GLOB>`، `--exclude <GLOB>` | تقييد مسارات البحث المتكرر أو استثناؤها. |
| `--color <MODE>`، `--line-buffered` | التحكم في تمييز ANSI أو تفريغ كل سطر للأنابيب. |
| `--threshold <0..1>`، `--score` | حدّ الصلة وإظهار الدرجة في الوضع الدلالي فقط. |
| `--ai`، `--ai-max-results <NUM>` | مخرجات مواقع مختصرة لوكلاء البرمجة وحدّها. |
| `--model <PATH>`، `--download-model`، `--offline` | استخدام ملف GGUF محلي، أو تنزيل النموذج المثبّت، أو منع الشبكة. |
| `--device <MODE>` | اختيار جهاز الاستدلال المحلي. |

لا يمكن الجمع بين `-E` و`-F`، و`-i` متاح في الوضعين المعجميين فقط. تقبل `--color` القيم `auto` أو `always` أو `never`، وتقبل `--device` القيم `auto` أو `cpu`. تُرفض `--threshold` و`--score` في الوضعين المعجميين حتى لا يغيّر خطأ مطبعي معنى الاستعلام بصمت. استخدم `--` قبل نمط أو مسار يبدأ بـ `-`.

يطبع `--ai` مواقع مختصرة فقط بصيغة `path:line`، بلا نص الأسطر المطابقة أو
ألوان ANSI أو الدرجات أو أسطر السياق. الحد الافتراضي هو 50 موقعًا عبر
التشغيل كله، ويمكن تغييره بواسطة `--ai-max-results`. بعدها يستطيع وكيل
البرمجة جلب نطاقات الأسطر الضيقة اللازمة فقط، مما يقلل استهلاك الرموز في
استدعاءات الأدوات.

## معيار رموز Codex

في معيار مضبوط من ثلاث مهام للبحث عن مواقع الشيفرة، استهلك `jgrep --ai -F`
رموز Codex إجمالية أقل بنسبة **16.4%** من `rg -F`. انخفض الإدخال غير المخزّن
مؤقتًا بنسبة 54.9%، وانخفض نص أداة البحث بنسبة 97.1%. هذه نتيجة تشغيل واحد
وليست ضمانًا عامًا. تطابقت المواقع في مهمتين؛ وفي المهمة الأوسع تخطّت إجابة
Codex في ذراع `rg` موقعًا كان موجودًا في خرج الأداة الخام.

راجع [المنهجية والقيود](benchmark/README.md)، و[ملخص النتيجة](benchmark/RESULTS-2026-09-22.md)، و[بيانات كل تشغيل](benchmark/results/2026-09-22.json)، و[سجلات التنفيذ الخام](benchmark/logs/2026-09-22/README.md).

## النموذج والخصوصية والحدود

يستخدم البحث الدلالي ملف **Qwen2.5-0.5B-Instruct GGUF Q8_0** الرسمي (نحو 676 ميجابايت). يثبّت البرنامج مراجعة النموذج وقيمة SHA-256 في المصدر، ويتحقق منه قبل وضعه ذريًا في ذاكرة التخزين المؤقت الخاصة بالمستخدم وإعادة استعماله. يبدأ التنزيل عند الحاجة الأولى للاستدلال الدلالي أو عند تشغيل `--download-model`.

مع سياق، يجهز `--download-model` ذاكرة التخزين المؤقت ثم يبحث؛ ومن دون سياق يكون أمر تنزيل فقط. يمنع `--offline` أي استخدام للشبكة ويفشل إذا لم يتوفر نموذج محلي صالح، ولا يمكن جمعه مع `--download-model`. يتيح `--model /path/to/model.gguf` استخدام ملف محلي موجود بدل تنزيل النموذج أو استبداله. بعد التنزيل الاختياري، تُعالج النصوص محليًا.

درجة الصلة هي `sigmoid(logit(Yes) - logit(No))` وليست احتمالًا مُعايرًا أو ضمانًا للصواب. قد يتأثر البحث الدلالي بالغموض والنفي واللغات والأسطر الطويلة والمحتوى العدائي، وقد يكون أبطأ بكثير من `grep` المعجمي بسبب الاستدلال المحلي لكل سطر. لا تعتمد عليه وحده في قرارات السلامة أو القانون أو الطب أو الأمن. لا يقتطع البرنامج بصمت سطرًا يتجاوز حد 4,096 رمزًا مميزًا لمُوجّه الاستدلال.

## الإدخال والإخراج والترخيص

يقرأ `jgrep` تدريجيًا ويطبع الأسطر المختارة بترتيب الإدخال. يدعم نص UTF-8 ونهايات LF أو CRLF والمسارات الموحّدة. عند البحث المتكرر لا يتبع وصلات المجلدات الرمزية؛ ويتخطى الملفات الثنائية المكتشفة مع تشخيص، بينما يعدّ ملفًا غير نصي مُسمّى صراحةً خطأً. تذهب التشخيصات وتقدم التنزيل إلى stderr.

مثل `grep`، تكون حالة الخروج `0` عند العثور على اختيار، و`1` عند عدم العثور عليه، و`2` عند الخطأ.

كود المشروع مرخّص بموجب **GPL-3.0-or-later**. راجع [LICENSE](LICENSE) و[NOTICE](NOTICE) و[THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md) قبل إعادة توزيع نسخة مبنية. نموذج Qwen الافتراضي أصل منفصل بترخيص Apache-2.0 ولا يُرخّص بصفته كود المشروع.
