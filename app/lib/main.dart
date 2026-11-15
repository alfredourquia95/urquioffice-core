import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:file_picker/file_picker.dart';
import 'package:path_provider/path_provider.dart';
import 'src/rust/api.dart' as rust_api;
import 'src/rust/frb_generated.dart';
import 'widgets/ribbon.dart';
import 'widgets/document_canvas.dart';
import 'widgets/side_panel_ia.dart';

void main() async {
  WidgetsFlutterBinding.ensureInitialized();
  // En Android el .so vive extraído en nativeLibraryDir del APK;
  // DynamicLibrary.open('lib<stem>.so') lo resuelve via el linker del sistema.
  // Usamos el loader por defecto de frb (stem rust_lib_urquioffice).
  await RustLib.init();
  runApp(const UrquiOfficeApp());
}

class UrquiOfficeApp extends StatelessWidget {
  const UrquiOfficeApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'UrquiOffice',
      debugShowCheckedModeBanner: false,
      theme: ThemeData(
        useMaterial3: true,
        colorScheme: ColorScheme.fromSeed(
          seedColor: const Color(0xFF1F4E79),
          brightness: Brightness.light,
        ),
        scaffoldBackgroundColor: const Color(0xFFE8EAED),
      ),
      home: const HomePage(),
    );
  }
}

class HomePage extends StatefulWidget {
  const HomePage({super.key});

  @override
  State<HomePage> createState() => _HomePageState();
}

class _HomePageState extends State<HomePage> {
  String estadoRust = 'Inicializando Rust...';
  String pestanaActiva = 'Inicio';
  bool panelIAVisible = true;

  @override
  void initState() {
    super.initState();
    _probarConexion();
  }

  Future<void> _probarConexion() async {
    try {
      final resultado = await rust_api.estadoNucleo();
      if (!mounted) return;
      setState(() {
        estadoRust = resultado.startsWith('OK')
            ? 'Nucleo Rust OK'
            : 'Error desde Rust: $resultado';
      });
    } catch (e) {
      if (!mounted) return;
      setState(() => estadoRust = 'Excepcion llamando a Rust: $e');
    }
    // Fase 6: arrancar el servidor IA local con la ruta real de la app.
    await _arrancarServidorIA();
  }

  Future<void> _arrancarServidorIA() async {
    try {
      final dir = await getApplicationDocumentsDirectory();
      final rutaDb = '${dir.path}/urquioffice.db';
      final r = await rust_api.arrancarServidorIa(
        rutaDb: rutaDb,
        secreto: 'urquioffice-local-secret',
      );
      if (!mounted) return;
      setState(() {
        estadoRust = '$estadoRust | $r';
      });
    } catch (e) {
      if (!mounted) return;
      setState(() => estadoRust = '$estadoRust | IA error: $e');
    }
  }

  @override
  Widget build(BuildContext context) {
    return CallbackShortcuts(
      bindings: {
        // Atajos de teclado globales (Fase 4: solo registrar, sin logica real)
        const SingleActivator(LogicalKeyboardKey.keyS, control: true): () => _accionGuardar(),
        const SingleActivator(LogicalKeyboardKey.keyO, control: true): () => _accionAbrir(),
        const SingleActivator(LogicalKeyboardKey.keyB, control: true): () => _accionNegrita(),
        const SingleActivator(LogicalKeyboardKey.keyI, control: true): () => _accionCursiva(),
        const SingleActivator(LogicalKeyboardKey.keyU, control: true): () => _accionSubrayar(),
      },
      child: Focus(
        autofocus: true,
        child: Scaffold(
          appBar: _buildAppBar(),
          body: Column(
            children: [
              Ribbon(
                pestanaActiva: pestanaActiva,
                onCambiarPestana: (p) => setState(() => pestanaActiva = p),
                onAccion: _onRibbonAccion,
              ),
              Expanded(
                child: Row(
                  children: [
                    Expanded(
                      child: DocumentCanvas(mensajeEstado: estadoRust),
                    ),
                    if (panelIAVisible)
                      SidePanelIA(
                        onCerrar: () => setState(() => panelIAVisible = false),
                        onRepintar: _repintarCanvas,
                      ),
                  ],
                ),
              ),
              _buildStatusBar(),
            ],
          ),
        ),
      ),
    );
  }

  PreferredSizeWidget _buildAppBar() {
    return AppBar(
      title: const Row(
        children: [
          Icon(Icons.description_outlined, size: 22),
          SizedBox(width: 8),
          Text('UrquiOffice', style: TextStyle(fontWeight: FontWeight.w600)),
          SizedBox(width: 12),
          Text(
            'Documento1.docx',
            style: TextStyle(fontSize: 13, fontWeight: FontWeight.w400),
          ),
        ],
      ),
      actions: [
        IconButton(
          tooltip: 'Abrir (Ctrl+O)',
          icon: const Icon(Icons.folder_open_outlined),
          onPressed: _accionAbrir,
        ),
        IconButton(
          tooltip: 'Guardar (Ctrl+S)',
          icon: const Icon(Icons.save_outlined),
          onPressed: _accionGuardar,
        ),
        IconButton(
          tooltip: panelIAVisible ? 'Ocultar panel IA' : 'Mostrar panel IA',
          icon: Icon(panelIAVisible ? Icons.chat_bubble : Icons.chat_bubble_outline),
          onPressed: () => setState(() => panelIAVisible = !panelIAVisible),
        ),
        const SizedBox(width: 8),
      ],
    );
  }

  Widget _buildStatusBar() {
    return Container(
      height: 26,
      padding: const EdgeInsets.symmetric(horizontal: 12),
      color: const Color(0xFF1F4E79),
      child: Row(
        children: [
          const Icon(Icons.circle, size: 8, color: Colors.greenAccent),
          const SizedBox(width: 6),
          Text(
            estadoRust,
            style: const TextStyle(color: Colors.white, fontSize: 12),
          ),
          const Spacer(),
          const Text(
            'Listo  |  Pagina 1 de 1  |  Palabras: 0',
            style: TextStyle(color: Colors.white70, fontSize: 12),
          ),
        ],
      ),
    );
  }

  // --- Acciones reales vía execute_command (Fase 5) ---
  Future<void> _ejecutarComando(String comando, String args) async {
    try {
      final r = await rust_api.executeCommand(commandName: comando, args: args);
      if (!mounted) return;
      setState(() => estadoRust = r.isEmpty ? 'OK: $comando' : 'OK: $comando -> $r');
    } catch (e) {
      if (!mounted) return;
      setState(() => estadoRust = 'Error en $comando: $e');
    }
  }

  Future<void> _accionGuardar() async {
    // Fase 5.4: SAF para elegir dónde guardar el .docx
    try {
      final resultado = await FilePicker.platform.saveFile(
        dialogTitle: 'Guardar documento',
        fileName: 'Documento1.docx',
        type: FileType.custom,
        allowedExtensions: ['docx'],
      );
      if (resultado == null) {
        if (mounted) setState(() => estadoRust = 'Guardar cancelado');
        return;
      }
      final r = await rust_api.fileGuardar(path: resultado);
      if (!mounted) return;
      setState(() => estadoRust = 'Guardado: $r');
    } catch (e) {
      if (!mounted) return;
      setState(() => estadoRust = 'Error al guardar: $e');
    }
  }

  Future<void> _accionAbrir() async {
    // Fase 5.4: SAF para elegir el .docx a abrir
    try {
      final resultado = await FilePicker.platform.pickFiles(
        dialogTitle: 'Abrir documento',
        type: FileType.custom,
        allowedExtensions: ['docx'],
      );
      if (resultado == null || resultado.files.isEmpty) {
        if (mounted) setState(() => estadoRust = 'Abrir cancelado');
        return;
      }
      final path = resultado.files.single.path;
      if (path == null) {
        if (mounted) setState(() => estadoRust = 'Ruta no disponible (usa almacenamiento propio)');
        return;
      }
      final r = await rust_api.fileAbrir(path: path);
      if (!mounted) return;
      setState(() => estadoRust = 'Abierto: $r');
    } catch (e) {
      if (!mounted) return;
      setState(() => estadoRust = 'Error al abrir: $e');
    }
  }

  void _accionNegrita() => _ejecutarComando('format.bold', '');
  void _accionCursiva() => _ejecutarComando('format.italic', '');
  void _accionSubrayar() => _ejecutarComando('format.underline', '');

  void _onRibbonAccion(String comando) {
    switch (comando) {
      case 'guardar':
        _accionGuardar();
      case 'abrir':
        _accionAbrir();
      case 'negrita':
        _accionNegrita();
      case 'cursiva':
        _accionCursiva();
      case 'subrayar':
        _accionSubrayar();
      case 'edit.undo':
        _ejecutarComando('edit.undo', '');
      case 'edit.redo':
        _ejecutarComando('edit.redo', '');
      case 'align.left':
        _ejecutarComando('align.left', '');
      case 'align.center':
        _ejecutarComando('align.center', '');
      case 'align.right':
        _ejecutarComando('align.right', '');
      case 'panel_ia':
        setState(() => panelIAVisible = !panelIAVisible);
      case 'ai.review':
        _accionRevisarOrtografia();
    }
  }

  /// Fase 6.4: revisa la ortografía del documento vía Rust (craft-proof)
  /// y muestra el resumen en la barra de estado.
  /// Repinta el canvas tras tool calls del agente IA (Fase 6.3):
  /// cambia el mensaje de estado para que DocumentCanvas recargue el dibujo.
  void _repintarCanvas() {
    setState(() {
      estadoRust = ' | documento actualizado por IA';
    });
  }

  Future<void> _accionRevisarOrtografia() async {
    try {
      final json = await rust_api.revisarOrtografia();
      final lista = jsonDecode(json) as List<dynamic>;
      if (!mounted) return;
      setState(() {
        estadoRust = lista.isEmpty
            ? 'Ortografía: sin errores'
            : 'Ortografía: ${lista.length} error(es). Primero: '
                '${lista[0]['palabra']} → ${(lista[0]['sugerencias'] as List).take(3).join(', ')}';
      });
    } catch (e) {
      if (!mounted) return;
      setState(() => estadoRust = 'Error revisando ortografía: $e');
    }
  }
}
