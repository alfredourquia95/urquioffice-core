import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
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

  void _accionGuardar() {
    // Fase 5.4 (SAF) pendiente: por ahora marcamos estado.
    setState(() => estadoRust = 'Guardar: pendiente SAF (fase 5.4)');
  }

  void _accionAbrir() {
    setState(() => estadoRust = 'Abrir: pendiente SAF (fase 5.4)');
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
    }
  }
}
