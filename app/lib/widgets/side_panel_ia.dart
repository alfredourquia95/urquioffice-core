import 'dart:async';

import 'package:flutter/material.dart';

import '../services/cliente_ia.dart';

/// Un mensaje del chat. [mio] = enviado por el usuario (alineado a la derecha).
class MensajeChat {
  MensajeChat({required this.texto, required this.mio});
  String texto;
  final bool mio;
}

/// Panel lateral derecho del agente de IA.
/// Fase 6: chat real contra el servidor local 127.0.0.1:31337 con streaming SSE.
class SidePanelIA extends StatefulWidget {
  final VoidCallback? onCerrar;

  const SidePanelIA({super.key, this.onCerrar});

  @override
  State<SidePanelIA> createState() => _SidePanelIAState();
}

class _SidePanelIAState extends State<SidePanelIA> {
  final _controlador = TextEditingController();
  final _cliente = ClienteIA();
  final _mensajes = <MensajeChat>[
    MensajeChat(
      texto: 'Hola, soy el asistente de UrquiOffice.',
      mio: false,
    ),
    MensajeChat(
      texto: 'Puedo resumir, traducir y revisar tu documento.',
      mio: false,
    ),
  ];
  final _historial = <Map<String, String>>[
    {
      'role': 'system',
      'content':
          'Eres el asistente de UrquiOffice, una suite ofimatica. Responde en '
          'español, de forma breve y directa. Ayuda a editar el documento del usuario.',
    },
  ];
  bool _esperando = false;
  int _hoverEnviar = 0;
  StreamSubscription<String>? _sub;

  @override
  void dispose() {
    _sub?.cancel();
    _controlador.dispose();
    super.dispose();
  }

  Future<void> _enviar() async {
    final texto = _controlador.text.trim();
    if (texto.isEmpty || _esperando) return;
    setState(() {
      _mensajes.add(MensajeChat(texto: texto, mio: true));
      _mensajes.add(MensajeChat(texto: '', mio: false)); // burbuja en curso
      _esperando = true;
      _controlador.clear();
    });
    _historial.add({'role': 'user', 'content': texto});

    final indiceRespuesta = _mensajes.length - 1;
    try {
      final stream = _cliente.chatStream(List.of(_historial));
      final buffer = StringBuffer();
      await for (final frag in stream) {
        buffer.write(frag);
        if (!mounted) break;
        setState(() {
          _mensajes[indiceRespuesta].texto = buffer.toString();
        });
      }
      final respuesta = buffer.toString();
      if (respuesta.isNotEmpty) {
        _historial.add({'role': 'assistant', 'content': respuesta});
      }
    } catch (e) {
      if (mounted) {
        setState(() {
          _mensajes[indiceRespuesta].texto = 'Error IA: $e';
        });
      }
    } finally {
      if (mounted) setState(() => _esperando = false);
    }
  }

  @override
  Widget build(BuildContext context) {
    return Container(
      width: 300,
      decoration: const BoxDecoration(
        color: Colors.white,
        border: Border(left: BorderSide(color: Color(0xFFC9CDD4))),
      ),
      child: Column(
        children: [
          // Encabezado del panel
          Container(
            height: 42,
            padding: const EdgeInsets.symmetric(horizontal: 12),
            decoration: const BoxDecoration(color: Color(0xFF1F4E79)),
            child: Row(
              children: [
                const Icon(Icons.auto_awesome, color: Colors.white, size: 18),
                const SizedBox(width: 8),
                const Text(
                  'Asistente IA',
                  style: TextStyle(
                    color: Colors.white,
                    fontWeight: FontWeight.w600,
                  ),
                ),
                const Spacer(),
                if (widget.onCerrar != null)
                  IconButton(
                    tooltip: 'Cerrar panel',
                    icon: const Icon(Icons.close, color: Colors.white, size: 18),
                    onPressed: widget.onCerrar,
                    padding: EdgeInsets.zero,
                    constraints: const BoxConstraints(),
                  ),
              ],
            ),
          ),
          // Lista de mensajes
          Expanded(
            child: ListView.builder(
              padding: const EdgeInsets.all(12),
              itemCount: _mensajes.length,
              itemBuilder: (context, i) {
                final m = _mensajes[i];
                return Align(
                  alignment: m.mio
                      ? Alignment.centerRight
                      : Alignment.centerLeft,
                  child: Container(
                    margin: const EdgeInsets.only(bottom: 8),
                    padding: const EdgeInsets.symmetric(
                      horizontal: 10,
                      vertical: 8,
                    ),
                    constraints: const BoxConstraints(maxWidth: 230),
                    decoration: BoxDecoration(
                      color: m.mio
                          ? const Color(0xFF1F4E79)
                          : const Color(0xFFF0F2F5),
                      borderRadius: BorderRadius.circular(10),
                    ),
                    child: Text(
                      m.texto.isEmpty && !m.mio && _esperando
                          ? '...'
                          : m.texto,
                      style: TextStyle(
                        fontSize: 13,
                        color: m.mio ? Colors.white : const Color(0xFF333333),
                      ),
                    ),
                  ),
                );
              },
            ),
          ),
          const Divider(height: 1),
          // Campo de texto + boton enviar
          Padding(
            padding: const EdgeInsets.all(10),
            child: Row(
              children: [
                Expanded(
                  child: TextField(
                    controller: _controlador,
                    enabled: !_esperando,
                    decoration: InputDecoration(
                      hintText: 'Escribe una instruccion...',
                      isDense: true,
                      contentPadding: const EdgeInsets.symmetric(
                        horizontal: 10,
                        vertical: 10,
                      ),
                      border: OutlineInputBorder(
                        borderRadius: BorderRadius.circular(8),
                        borderSide: const BorderSide(color: Color(0xFFC9CDD4)),
                      ),
                    ),
                    onSubmitted: (_) => _enviar(),
                  ),
                ),
                const SizedBox(width: 8),
                MouseRegion(
                  onEnter: (_) => setState(() => _hoverEnviar = 1),
                  onExit: (_) => setState(() => _hoverEnviar = 0),
                  child: IconButton(
                    tooltip: 'Enviar',
                    icon: Icon(
                      Icons.send,
                      color: _hoverEnviar == 1
                          ? const Color(0xFF0D3B66)
                          : const Color(0xFF1F4E79),
                    ),
                    onPressed: _enviar,
                  ),
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }
}
