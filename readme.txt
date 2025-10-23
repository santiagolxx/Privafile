Privafile

## Se busca que este proyecto sea modular, por lo cual cuenta con una libreria nucleo la cual tiene las funciones basicas para poder construir servidores para privafile.

Privafile es un proyecto que se basa en construir un servicio de drive open source, libre, modular y facil de usar pero con un rendimiento alto.
Por eso preferimos el uso de lenguajes de bajo nivel sobre interpretados.
La idea es permitir guardar archivos de forma segura y privada sin que nadie mas que vos podas leer tus archivos, eso lo logramos a traves de criptografia avanzada tanto como simetrica y asimetrica, derivacion de claves y muchas curvas elipticas.

por ahora privafile esta organizado asi:

/src
- privamod (Modulos nucleo compartido entre servidores)
- servers (Servidores en si, por ahora tcp y http)
- Crypto (Librerias de criptografia)

Por ahora se cuenta con los siguientes

HTTP/S (Contruido con Rocket)
TCP/UDP (Proximamente)

---------
QA

Q: Donde se encriptan los archivos?
A: Usamos STProtocol el cual es un protocolo de transferencia hecho en casa el cual permite tanto encriptacion del lado del cliente (Z-K) y desde el servidor (Por default se encripta desde el cliente)

Q: Como son los archivos en si?
A: Los archivos se guardan de blobs STFiles. Los cuales estan organizados asi:
Clave AES-256GCM encriptada por clave RSA del cliente
Metadata encriptada de privafile (De quien es el archivo, cuando se subio etc)
Archivo encriptado en si
Metadata original encriptada

Debido a que STP usa padding se pueden acceder a ciertos tipos de datos sin la necesidad de desencriptar todo el STFile por lo cual el archivo original nunca se carga en ram en nuestros servidores.
