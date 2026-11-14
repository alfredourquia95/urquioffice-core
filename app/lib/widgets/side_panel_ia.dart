import 'package:flutter/material.dart';

/// Panel lateral derecho del agente de IA.
/// Fase 4: campo de texto y boton enviar; el boton aun no hace nada.
class SidePanelIA extends StatefulWidget {
  final VoidCallback? onCerrar;

  const SidePanelIA({super.key, this.onCerrar});

  @override
  State<SidePanelIA> createState() => _SidePanelIAState();
}

class _SidePanelIAState extends State<SidePanelIA> {
  final _controlador = TextEditingController();
  final _mensajes = <String>[
    'Hola, soy el asistente de UrquiOffice.',
    'Puedo resumir, traducir y revisar tu documento.',
  ];
  int _hoverEnviar = 0;

  @override
  void dispose() {
    _controlador.dispose();
    super.dispose();
  }

  void _enviar() {
    // Fase 4: el boton no hace nada real; solo anade el texto a la lista local.
    final texto = _controlador.text.trim();
    if (texto.isEmpty) return;
    setState(() {
      _mensajes.add(texto);
      _controlador.clear();
    });
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
            decoration: const BoxDecoration(
              color: Color(0xFF1F4E79),
              borderRadius: BorderRadius.only(topRight: Radius.circular(0)),
            ),
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
                final mio = i.isOdd;
                return Align(
                  alignment: mio ? Alignment.centerRight : Alignment.centerLeft,
                  child: Container(
                    margin: const EdgeInsets.only(bottom: 8),
                    padding: const EdgeInsets.symmetric(
                      horizontal: 10,
                      vertical: 8,
                    ),
                    constraints: const BoxConstraints(maxWidth: 230),
                    decoration: BoxDecoration(
                      color: mio
                          ? const Color(0xFF1F4E79)
                          : const Color(0xFFF0F2F5),
                      borderRadius: BorderRadius.circular(10),
                    ),
                    child: Text(
                      _mensajes[i],
                      style: TextStyle(
                        fontSize: 13,
                        color: mio ? Colors.white : const Color(0xFF333333),
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
