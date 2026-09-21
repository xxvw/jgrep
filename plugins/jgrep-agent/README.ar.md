# إضافة Codex `jgrep-agent`

تزوّد هذه الإضافة Codex بإرشادات لاستخدام `jgrep` كأول بحث عن مواضع الشيفرة.
وهي تفضّل المخرجات المختصرة كي يقرأ الوكيل نطاقات المصدر المطلوبة فقط. ثبّت
أيضًا الملف التنفيذي `jgrep`؛ فالإضافة لا تتضمنه.

## التثبيت

من طرفية يتوفر فيها سطر أوامر Codex، أضف سوق الإضافات الخاص بالمشروع ثم ثبّت
الإضافة:

```sh
codex plugin marketplace add xxvw/localjev-grep --ref main --sparse .agents/plugins --sparse plugins/jgrep-agent && codex plugin add jgrep-agent@localjev-grep
```

ابدأ جلسة Codex جديدة لتحميل الإضافة. ثبّت `jgrep` باتباع
[ملف README الرئيسي](https://github.com/xxvw/localjev-grep/blob/main/README.md) و[دليل التثبيت ودمج الوكلاء](https://github.com/xxvw/localjev-grep/blob/main/docs/installation-and-agents.md).

## بحث مختصر للوكلاء

ابدأ بالخيار `--ai`. فهو يطبع مواضع `المسار:السطر` فقط، من دون نص المصدر،
ويقيّد المخرجات افتراضيًا إلى 50 نتيجة:

```sh
# بحث دلالي عن سلوك
jgrep --ai --ai-max-results 25 -r --include '*.rs' \
  'أين تتم معالجة حالات فشل المصادقة' src/

# معرّف أو نص معروف: مطابقة حرفية لا تحتاج إلى نموذج
jgrep --ai --ai-max-results 25 -r -F 'validate_session' src/

# نمط معروف: تعبير Rust نمطي لا يحتاج إلى نموذج
jgrep --ai --ai-max-results 25 -r -E 'validate_(session|token)' src/
```

حلّل دائمًا اللاحقة النهائية المطابقة لـ `:[0-9]+$` في النتيجة. فقد يحتوي مسار Windows
على نقطتين قبله، مثل `C:\work\src\auth.rs:57`. اقرأ النطاقات الضيقة المشار
إليها فقط قبل توسيع البحث.

إذا نبّه stderr إلى بلوغ الحد، فالنتيجة غير مكتملة: ضيّق الاستعلام أو الدليل
قبل زيادة `--ai-max-results`. استخدم `rg` إذا لم يكن `jgrep` مثبّتًا أو إذا
تعذّر التعبير عن المهمة بالبحث المختصر.
