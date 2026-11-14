import 'package:flutter/material.dart';
import 'src/rust/api.dart' as rust_api;
import 'src/rust/frb_generated.dart';

void main() async {
  WidgetsFlutterBinding.ensureInitialized();
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
  String estado = 'Inicializando Rust...';

  @override
  void initState() {
    super.initState();
    _probarConexion();
  }

  Future<void> _probarConexion() async {
    try {
      final resultado = await rust_api.estadoNucleo();
      setState(() {
        estado = resultado.startsWith('OK')
            ? 'Conexión con Rust establecida\n\n$resultado'
            : 'Error desde Rust: $resultado';
      });
    } catch (e) {
      setState(() => estado = 'Excepción llamando a Rust: $e');
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('UrquiOffice')),
      body: Center(
        child: Padding(
          padding: const EdgeInsets.all(24),
          child: SelectableText(
            estado,
            style: const TextStyle(fontFamily: 'monospace', fontSize: 14),
          ),
        ),
      ),
    );
  }
}
