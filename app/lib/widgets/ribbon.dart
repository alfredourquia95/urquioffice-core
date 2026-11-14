import 'package:flutter/material.dart';

/// Barra superior estilo Ribbon con pestanas: Inicio, Insertar, Diseno, IA.
/// Fase 4: solo visual, con hover y navegacion por teclado entre pestanas.
class Ribbon extends StatefulWidget {
  final String pestanaActiva;
  final ValueChanged<String> onCambiarPestana;
  final ValueChanged<String> onAccion;

  const Ribbon({
    super.key,
    required this.pestanaActiva,
    required this.onCambiarPestana,
    required this.onAccion,
  });

  @override
  State<Ribbon> createState() => _RibbonState();
}

class _RibbonState extends State<Ribbon> {
  static const _pestanas = ['Inicio', 'Insertar', 'Diseno', 'IA'];
  int _hoverTab = -1;
  int _hoverBoton = -1;

  @override
  Widget build(BuildContext context) {
    return Container(
      decoration: const BoxDecoration(
        color: Colors.white,
        border: Border(bottom: BorderSide(color: Color(0xFFC9CDD4))),
      ),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          // Fila de pestanas
          SizedBox(
            height: 38,
            child: Row(
              children: [
                const SizedBox(width: 12),
                for (var i = 0; i < _pestanas.length; i++)
                  _buildPestana(_pestanas[i], i),
                const Spacer(),
                _buildBotonRibbon(Icons.search, 'Buscar', -100, () {}),
                const SizedBox(width: 8),
              ],
            ),
          ),
          const Divider(height: 1, color: Color(0xFFC9CDD4)),
          // Fila de botones segun la pestana activa
          SizedBox(
            height: 84,
            child: _buildBotonesPestana(),
          ),
        ],
      ),
    );
  }

  Widget _buildPestana(String nombre, int index) {
    final activa = widget.pestanaActiva == nombre;
    final hover = _hoverTab == index;
    return MouseRegion(
      onEnter: (_) => setState(() => _hoverTab = index),
      onExit: (_) => setState(() => _hoverTab = -1),
      child: GestureDetector(
        onTap: () => widget.onCambiarPestana(nombre),
        child: Container(
          padding: const EdgeInsets.symmetric(horizontal: 16),
          decoration: BoxDecoration(
            color: activa
                ? const Color(0xFFDCE6F1)
                : hover
                    ? const Color(0xFFF0F4F9)
                    : Colors.transparent,
            border: activa
                ? const Border(
                    bottom: BorderSide(color: Color(0xFF1F4E79), width: 2),
                  )
                : null,
          ),
          alignment: Alignment.center,
          child: Text(
            nombre,
            style: TextStyle(
              fontWeight: activa ? FontWeight.w600 : FontWeight.w400,
              color: activa ? const Color(0xFF1F4E79) : const Color(0xFF333333),
            ),
          ),
        ),
      ),
    );
  }

  Widget _buildBotonesPestana() {
    final grupos = _botonesPorPestana[widget.pestanaActiva] ?? [];
    return ListView.separated(
      scrollDirection: Axis.horizontal,
      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
      itemCount: grupos.length,
      separatorBuilder: (_, index) => const VerticalDivider(width: 24, color: Color(0xFFDDE1E6)),
      itemBuilder: (context, gi) {
        final grupo = grupos[gi];
        return Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            for (var bi = 0; bi < grupo.botones.length; bi++)
              _buildBotonRibbon(
                grupo.botones[bi].icono,
                grupo.botones[bi].etiqueta,
                gi * 100 + bi,
                () => widget.onAccion(grupo.botones[bi].comando),
              ),
            if (grupo.titulo != null) ...[
              const SizedBox(width: 6),
              Align(
                alignment: Alignment.bottomCenter,
                child: Text(
                  grupo.titulo!,
                  style: const TextStyle(fontSize: 10, color: Color(0xFF777777)),
                ),
              ),
            ],
          ],
        );
      },
    );
  }

  Widget _buildBotonRibbon(IconData icono, String etiqueta, int id, VoidCallback onTap) {
    final hover = _hoverBoton == id;
    return MouseRegion(
      onEnter: (_) => setState(() => _hoverBoton = id),
      onExit: (_) => setState(() => _hoverBoton = -1),
      child: InkWell(
        onTap: onTap,
        borderRadius: BorderRadius.circular(6),
        child: Container(
          width: 64,
          padding: const EdgeInsets.symmetric(vertical: 4),
          decoration: BoxDecoration(
            color: hover ? const Color(0xFFEAF0F8) : Colors.transparent,
            borderRadius: BorderRadius.circular(6),
          ),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              Icon(icono, size: 24, color: const Color(0xFF1F4E79)),
              const SizedBox(height: 2),
              Text(
                etiqueta,
                style: const TextStyle(fontSize: 11, color: Color(0xFF333333)),
                textAlign: TextAlign.center,
              ),
            ],
          ),
        ),
      ),
    );
  }

  static final Map<String, List<_GrupoBotones>> _botonesPorPestana = {
    'Inicio': [
      _GrupoBotones('Portapapeles', [
        _BotonRibbon(Icons.content_paste, 'Pegar', 'pegar'),
        _BotonRibbon(Icons.content_copy, 'Copiar', 'copiar'),
        _BotonRibbon(Icons.content_cut, 'Cortar', 'cortar'),
      ]),
      _GrupoBotones('Fuente', [
        _BotonRibbon(Icons.format_bold, 'Negrita', 'negrita'),
        _BotonRibbon(Icons.format_italic, 'Cursiva', 'cursiva'),
        _BotonRibbon(Icons.format_underlined, 'Subrayar', 'subrayar'),
      ]),
      _GrupoBotones('Parrafo', [
        _BotonRibbon(Icons.format_align_left, 'Izquierda', 'align.left'),
        _BotonRibbon(Icons.format_align_center, 'Centrar', 'align.center'),
        _BotonRibbon(Icons.format_align_right, 'Derecha', 'align.right'),
      ]),
      _GrupoBotones(null, [
        _BotonRibbon(Icons.undo, 'Deshacer', 'edit.undo'),
        _BotonRibbon(Icons.redo, 'Rehacer', 'edit.redo'),
      ]),
    ],
    'Insertar': [
      _GrupoBotones('Paginas', [
        _BotonRibbon(Icons.note_add, 'Pagina en blanco', 'insert.page'),
      ]),
      _GrupoBotones('Tablas', [
        _BotonRibbon(Icons.table_chart, 'Tabla', 'insert.table'),
      ]),
      _GrupoBotones('Ilustraciones', [
        _BotonRibbon(Icons.image, 'Imagen', 'insert.image'),
        _BotonRibbon(Icons.insert_link, 'Hipervinculo', 'insert.link'),
      ]),
    ],
    'Diseno': [
      _GrupoBotones('Tema', [
        _BotonRibbon(Icons.palette, 'Colores', 'design.colors'),
        _BotonRibbon(Icons.text_fields, 'Fuentes', 'design.fonts'),
      ]),
      _GrupoBotones('Pagina', [
        _BotonRibbon(Icons.crop, 'Margenes', 'design.margins'),
        _BotonRibbon(Icons.straighten, 'Tamano', 'design.size'),
        _BotonRibbon(Icons.rotate_right, 'Orientacion', 'design.orientation'),
      ]),
    ],
    'IA': [
      _GrupoBotones('Asistente', [
        _BotonRibbon(Icons.auto_awesome, 'Resumir', 'ai.summarize'),
        _BotonRibbon(Icons.translate, 'Traducir', 'ai.translate'),
        _BotonRibbon(Icons.spellcheck, 'Revisar', 'ai.review'),
      ]),
      _GrupoBotones(null, [
        _BotonRibbon(Icons.chat_bubble, 'Abrir panel IA', 'panel_ia'),
      ]),
    ],
  };
}

class _BotonRibbon {
  final IconData icono;
  final String etiqueta;
  final String comando;
  const _BotonRibbon(this.icono, this.etiqueta, this.comando);
}

class _GrupoBotones {
  final String? titulo;
  final List<_BotonRibbon> botones;
  const _GrupoBotones(this.titulo, this.botones);
}
