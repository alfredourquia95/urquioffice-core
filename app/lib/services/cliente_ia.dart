import 'dart:async';
import 'dart:convert';

import 'package:http/http.dart' as http;

/// Cliente del servidor de IA local de UrquiOffice (127.0.0.1:31337).
/// Fase 6: habla con POST /v1/chat/completions (compatible OpenAI) y
/// consume la respuesta como stream SSE (data: {...}\n\n ...  es el formato del servidor local (axum), NO OpenAI puro: cada chunk
/// llega como evento SSE con JSON {"choices":[{"delta":{"content":"..."}}]}.
class ClienteIA {
  ClienteIA({this.baseUrl = 'http://127.0.0.1:31337', this.modelo = 'urquioffice-local'});

  final String baseUrl;
  final String modelo;

  /// Envia [mensajes] (lista de {'role','content'}) y devuelve un stream
  /// de fragmentos de texto (delta.content) a medida que llegan.
  Stream<String> chatStream(List<Map<String, String>> mensajes) async* {
    final uri = Uri.parse('$baseUrl/v1/chat/completions');
    final request = http.Request('POST', uri)
      ..headers['Content-Type'] = 'application/json'
      ..body = jsonEncode({
        'model': modelo,
        'stream': true,
        'messages': mensajes,
      });

    final response = await http.Client().send(request);
    if (response.statusCode != 200) {
      final cuerpo = await response.stream.bytesToString();
      throw Exception('IA local respondio ${response.statusCode}: $cuerpo');
    }

    // Parseo SSE: lineas "data: {...}" separadas por linea en blanco.
    final buffer = StringBuffer();
    await for (final chunk in response.stream.transform(utf8.decoder)) {
      buffer.write(chunk);
      // Procesa todas las lineas completas disponibles.
      while (true) {
        final datos = _extraerLineaCompleta(buffer);
        if (datos == null) break;
        if (!datos.startsWith('data: ')) continue;
        final payload = datos.substring(6).trim();
        if (payload == '[DONE]') return;
        try {
          final json = jsonDecode(payload) as Map<String, dynamic>;
          final choices = json['choices'] as List<dynamic>?;
          if (choices == null || choices.isEmpty) continue;
          final delta = (choices[0] as Map<String, dynamic>)['delta']
              as Map<String, dynamic>?;
          final content = delta?['content'] as String?;
          if (content != null && content.isNotEmpty) {
            yield content;
          }
        } catch (_) {
          // Ignora chunks no JSON (keep-alive, etc.).
        }
      }
    }
  }

  /// Extrae y devuelve la primera linea completa (terminada en \n) del buffer,
  /// o null si aun no hay ninguna. Consume la linea del buffer.
  static String? _extraerLineaCompleta(StringBuffer buffer) {
    final s = buffer.toString();
    final idx = s.indexOf('\n');
    if (idx < 0) return null;
    final linea = s.substring(0, idx).replaceAll('\r', '');
    buffer.clear();
    buffer.write(s.substring(idx + 1));
    return linea;
  }
}
