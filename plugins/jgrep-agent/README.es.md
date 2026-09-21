# Complemento jgrep-agent para Codex

jgrep-agent enseña a los agentes de programación a localizar primero el código
con la salida compacta de jgrep. Así se revisan únicamente los rangos de
origen pertinentes y se reduce el volumen de las llamadas a herramientas.

## Instalación

En Codex, añada el marketplace local de este repositorio e instale el plugin:

~~~sh
codex plugin marketplace add xxvw/jgrep --ref main --sparse .agents/plugins --sparse plugins/jgrep-agent && codex plugin add jgrep-agent@jgrep
~~~

Inicie una sesión nueva de Codex después de instalarlo. Instale después el
ejecutable jgrep con las instrucciones del
[README principal](https://github.com/xxvw/jgrep/blob/main/README.md) o de la
[guía de instalación y agentes](https://github.com/xxvw/jgrep/blob/main/docs/installation-and-agents.md).

## Flujo de trabajo del agente

Use --ai como primera búsqueda. Cada coincidencia es un localizador compacto
sin texto de origen:

~~~sh
jgrep --ai --ai-max-results 25 -r -F 'validate_session' src/
jgrep --ai --ai-max-results 25 -r -E 'TODO|FIXME' src/
jgrep --ai --ai-max-results 25 -r --include '*.rs' \
  'dónde se gestionan los fallos de autenticación' src/
~~~

Analice cada registro por el sufijo final `:LINE`, es decir, por los dos puntos
finales seguidos de un número decimal. No use los primeros dos puntos: una ruta
de Windows puede comenzar, por ejemplo, con `C:\`. `-:LINE` identifica la entrada
estándar de una canalización reproducible.

-F sirve para símbolos o texto conocidos y -E para patrones de expresión
regular; ambos son búsquedas léxicas precisas y no cargan el modelo. Use la
búsqueda semántica de --ai para describir comportamiento o conceptos.

El máximo predeterminado es 50 localizadores. Si jgrep informa en stderr que
se alcanzó el máximo, el resultado está incompleto: limite la consulta o el
directorio antes de aumentar --ai-max-results. Tras obtener los localizadores,
lea solo los rangos de líneas indicados.

Si jgrep no está instalado o la búsqueda compacta no puede expresar la
consulta, use rg como alternativa:

~~~sh
rg -n -F 'validate_session' src/
~~~
