# Codex प्लगइन `jgrep-agent`

यह प्लगइन Codex को कोड में स्थान खोजने के लिए पहले `jgrep` इस्तेमाल करने के
निर्देश देता है। यह संक्षिप्त आउटपुट को प्राथमिकता देता है, इसलिए एजेंट केवल
ज़रूरी स्रोत-सीमाएँ पढ़ता है। `jgrep` निष्पादन योग्य को भी इंस्टॉल करें; प्लगइन
में वह शामिल नहीं है।

## इंस्टॉल करें

जिस टर्मिनल में Codex CLI उपलब्ध हो, उसमें प्रोजेक्ट का marketplace जोड़ें और
प्लगइन इंस्टॉल करें:

```sh
codex plugin marketplace add xxvw/localjev-grep --ref main --sparse .agents/plugins --sparse plugins/jgrep-agent && codex plugin add jgrep-agent@localjev-grep
```

प्लगइन लोड करने के लिए नई Codex session शुरू करें। `jgrep` इंस्टॉल करने के लिए
[मुख्य README](https://github.com/xxvw/localjev-grep/blob/main/README.md) और
[इंस्टॉलेशन व एजेंट इंटीग्रेशन गाइड](https://github.com/xxvw/localjev-grep/blob/main/docs/installation-and-agents.md)
का पालन करें।

## एजेंटों के लिए संक्षिप्त खोज

`--ai` से शुरुआत करें। यह स्रोत-पाठ के बिना केवल `path:line` स्थान प्रिंट करता
है और डिफ़ॉल्ट रूप से आउटपुट को 50 परिणामों तक सीमित रखता है:

```sh
# किसी व्यवहार के लिए अर्थ-आधारित खोज
jgrep --ai --ai-max-results 25 -r --include '*.rs' \
  'जहाँ प्रमाणीकरण विफलताओं को संभाला जाता है' src/

# ज्ञात identifier या पाठ: मॉडल के बिना literal मिलान
jgrep --ai --ai-max-results 25 -r -F 'validate_session' src/

# ज्ञात pattern: मॉडल के बिना Rust regular expression
jgrep --ai --ai-max-results 25 -r -E 'validate_(session|token)' src/
```

हर परिणाम के अंतिम `:[0-9]+$` suffix को हमेशा पार्स करें। Windows path
में इससे पहले colon हो सकता है, जैसे `C:\work\src\auth.rs:57`। खोज को
बढ़ाने से पहले केवल बताए गए छोटे source ranges पढ़ें।

यदि stderr बताए कि परिणाम सीमा पहुँच गई है, तो परिणाम अधूरा है: पहले query या
directory को सीमित करें, फिर ही `--ai-max-results` बढ़ाएँ। यदि `jgrep` इंस्टॉल
न हो या संक्षिप्त खोज कार्य को व्यक्त न कर सके, तो `rg` इस्तेमाल करें।
