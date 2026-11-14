import 'package:flutter/material.dart';

/// Lienzo del documento. Fase 4: dibuja la hoja blanca con CustomPainter
/// y texto estatico de prueba; aun no edita nada real.
class DocumentCanvas extends StatelessWidget {
  final String mensajeEstado;

  const DocumentCanvas({super.key, this.mensajeEstado = ''});

  @override
  Widget build(BuildContext context) {
    return Container(
      color: const Color(0xFFE8EAED),
      child: LayoutBuilder(
        builder: (context, constraints) {
          // Tamano de hoja A4 escalado para caber en el area visible
          const aspect = 210 / 297; // ancho / alto A4
          var ancho = constraints.maxWidth * 0.62;
          var alto = ancho / aspect;
          if (alto > constraints.maxHeight * 0.92) {
            alto = constraints.maxHeight * 0.92;
            ancho = alto * aspect;
          }
          return SingleChildScrollView(
            child: Center(
              child: Padding(
                padding: const EdgeInsets.symmetric(vertical: 24),
                child: SizedBox(
                  width: ancho,
                  height: alto,
                  child: CustomPaint(
                    painter: _HojaPainter(),
                    child: Padding(
                      padding: EdgeInsets.all(ancho * 0.08),
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          Text(
                            'Documento de prueba',
                            style: TextStyle(
                              fontSize: ancho * 0.045,
                              fontWeight: FontWeight.bold,
                              color: Colors.black87,
                            ),
                          ),
                          SizedBox(height: ancho * 0.03),
                          Text(
                            'Este es un texto estatico de prueba para la Fase 4. '
                            'Aqui ira el contenido real del documento cuando el '
                            'motor de Rust envie la lista de dibujo.',
                            style: TextStyle(
                              fontSize: ancho * 0.028,
                              color: Colors.black54,
                              height: 1.4,
                            ),
                          ),
                          const Spacer(),
                          if (mensajeEstado.isNotEmpty)
                            Text(
                              mensajeEstado,
                              style: TextStyle(
                                fontSize: ancho * 0.022,
                                color: const Color(0xFF1F4E79),
                                fontFamily: 'monospace',
                              ),
                            ),
                        ],
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
  @override
  void paint(Canvas canvas, Size size) {
    // Sombra de la hoja
    final sombra = Paint()
      ..color = Colors.black.withValues(alpha: 0.18)
      ..maskFilter = const MaskFilter.blur(BlurStyle.normal, 6);
    canvas.drawRect(
      Rect.fromLTWH(4, 6, size.width, size.height),
      sombra,
    );
    // Hoja blanca con borde
    final hoja = Paint()..color = Colors.white;
    canvas.drawRect(Rect.fromLTWH(0, 0, size.width, size.height), hoja);
    final borde = Paint()
      ..color = const Color(0xFFB9BEC7)
      ..style = PaintingStyle.stroke;
    canvas.drawRect(Rect.fromLTWH(0, 0, size.width, size.height), borde);
  }

  @override
  bool shouldRepaint(covariant CustomPainter oldDelegate) => false;
}
