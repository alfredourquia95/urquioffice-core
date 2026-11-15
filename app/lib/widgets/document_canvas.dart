import 'dart:convert';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import '../src/rust/api.dart' as rust_api;

/// Una línea de dibujo recibida desde Rust (obtener_dibujo).
class DrawLine {
  final double x;
  final double y;
  final String text;
  final bool bold;
  const DrawLine({required this.x, required this.y, required this.text, required this.bold});

  factory DrawLine.fromJson(Map<String, dynamic> j) => DrawLine(
        x: (j['x'] as num).toDouble(),
        y: (j['y'] as num).toDouble(),
        text: j['text'] as String,
        bold: j['bold'] as bool,
      );
}

/// Lienzo del documento (Fase 5): editable. Cada tecla envía `text.insert`
/// a Rust vía execute_command y luego refresca la lista de dibujo con
/// obtener_dibujo() para repintar con CustomPainter.
class DocumentCanvas extends StatefulWidget {
  final String mensajeEstado;

  const DocumentCanvas({super.key, this.mensajeEstado = ''});

  @override
  State<DocumentCanvas> createState() => _DocumentCanvasState();
}

class _DocumentCanvasState extends State<DocumentCanvas> {
  final _focusNode = FocusNode();
  final List<DrawLine> _lineas = [];

  @override
  void initState() {
    super.initState();
    _refrescarDibujo();
  }

  @override
  void didUpdateWidget(covariant DocumentCanvas oldWidget) {
    super.didUpdateWidget(oldWidget);
    // Si cambió el mensaje de estado (p. ej. tras tool calls del agente IA),
    // forzamos recarga del dibujo desde Rust.
    if (oldWidget.mensajeEstado != widget.mensajeEstado) {
      _refrescarDibujo();
    }
  }

  @override
  void dispose() {
    _focusNode.dispose();
    super.dispose();
  }

  Future<void> _refrescarDibujo() async {
    try {
      final json = await rust_api.obtenerDibujo();
      final lista = (jsonDecode(json) as List<dynamic>)
          .map((e) => DrawLine.fromJson(e as Map<String, dynamic>))
          .toList();
      if (!mounted) return;
      setState(() {
        _lineas
          ..clear()
          ..addAll(lista);
      });
    } catch (_) {
      // Si Rust aún no tiene documento, el canvas queda en blanco.
    }
  }

  Future<void> _ejecutar(String comando, String args) async {
    try {
      await rust_api.executeCommand(commandName: comando, args: args);
    } catch (_) {
      // Fase 5: los errores se muestran en la barra de estado del padre más adelante.
    }
    await _refrescarDibujo();
  }

  void _onKey(KeyEvent event) {
    if (event is! KeyDownEvent) return;
    final ch = event.character;
    if (ch != null && ch.isNotEmpty && ch.codeUnitAt(0) >= 32) {
      _ejecutar('text.insert', jsonEncode({'text': ch}));
    } else if (event.logicalKey == LogicalKeyboardKey.backspace) {
      _ejecutar('edit.backspace', '');
    } else if (event.logicalKey == LogicalKeyboardKey.enter) {
      _ejecutar('text.insert', jsonEncode({'text': '\n'}));
    }
  }

  @override
  Widget build(BuildContext context) {
    return Container(
      color: const Color(0xFFE8EAED),
      child: LayoutBuilder(
        builder: (context, constraints) {
          const aspect = 210 / 297; // ancho / alto A4
          var ancho = constraints.maxWidth * 0.62;
          var alto = ancho / aspect;
          if (alto > constraints.maxHeight * 0.92) {
            alto = constraints.maxHeight * 0.92;
            ancho = alto * aspect;
          }
          // Escala: las coordenadas de Rust son en puntos A4 (595x842 aprox);
          // las adaptamos al ancho de la hoja en pantalla.
          const anchoHojaPt = 595.0;
          final escala = ancho / anchoHojaPt;
          return SingleChildScrollView(
            child: Center(
              child: Padding(
                padding: const EdgeInsets.symmetric(vertical: 24),
                child: SizedBox(
                  width: ancho,
                  height: alto,
                  child: CustomPaint(
                    painter: _HojaPainter(lineas: _lineas, escala: escala),
                    child: KeyboardListener(
                      focusNode: _focusNode,
                      autofocus: true,
                      onKeyEvent: _onKey,
                      child: GestureDetector(
                        onTap: () => _focusNode.requestFocus(),
                        child: const SizedBox.expand(),
                      ),
                    ),
                  ),
                ),
              ),
            ),
          );
        },
      ),
    );
  }
}

class _HojaPainter extends CustomPainter {
  final List<DrawLine> lineas;
  final double escala;
  _HojaPainter({required this.lineas, required this.escala});

  @override
  void paint(Canvas canvas, Size size) {
    // Sombra de la hoja
    final sombra = Paint()
      ..color = Colors.black.withValues(alpha: 0.18)
      ..maskFilter = const MaskFilter.blur(BlurStyle.normal, 6);
    canvas.drawRect(Rect.fromLTWH(4, 6, size.width, size.height), sombra);
    // Hoja blanca con borde
    canvas.drawRect(Rect.fromLTWH(0, 0, size.width, size.height), Paint()..color = Colors.white);
    canvas.drawRect(
      Rect.fromLTWH(0, 0, size.width, size.height),
      Paint()
        ..color = const Color(0xFFB9BEC7)
        ..style = PaintingStyle.stroke,
    );
    // Texto real del documento vindo de Rust
    for (final l in lineas) {
      final estilo = TextStyle(
        color: Colors.black87,
        fontSize: 14 * escala,
        fontFamily: 'Liberation Sans',
        fontWeight: l.bold ? FontWeight.bold : FontWeight.normal,
        height: 1.2,
      );
      final tp = TextPainter(
        text: TextSpan(text: l.text, style: estilo),
        textDirection: TextDirection.ltr,
      )..layout();
      tp.paint(canvas, Offset(l.x * escala, l.y * escala));
    }
  }

  @override
  bool shouldRepaint(covariant _HojaPainter old) => old.lineas != lineas || old.escala != escala;
}
