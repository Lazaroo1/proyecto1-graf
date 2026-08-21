# Proyecto 1 — Ray Caster en Rust

Ray caster en primera persona desarrollado en Rust para el curso de Gráficas por Computadora. El jugador recorre un laberinto texturizado, encuentra la meta marcada en el minimapa y completa el nivel sin atravesar paredes.

El render usa DDA por columna, corre en una ventana de `minifb` y combina texturas, iluminación por distancia, minimapa, música, efectos de sonido y entrada simultánea de teclado, mouse y control.

## Requisitos

- Rust estable con Cargo: <https://rustup.rs/>
- Una salida de audio para escuchar música y efectos (el juego sigue funcionando si no está disponible).
- Control compatible con `gilrs` opcional; no es necesario conectarlo para iniciar.

## Compilar y ejecutar

Desde la raíz del repositorio:

```bash
cargo run
```

Para compilar con optimizaciones:

```bash
cargo run --release
```

Para ejecutar la suite de pruebas:

```bash
cargo test
```

## Objetivo

Desde la pantalla de bienvenida, iniciá la partida y atravesá el laberinto hasta llegar a la meta verde. La meta también aparece como una bandera en el minimapa de la esquina superior derecha. Al alcanzarla se muestra la pantalla de éxito y se puede volver al menú.

## Controles

### Teclado y mouse

| Entrada | Acción |
| --- | --- |
| `W` / `S` | Avanzar y retroceder según la dirección de la cámara |
| `A` / `D` | Desplazamiento lateral |
| Flechas izquierda/derecha | Rotar la cámara |
| Mouse horizontal | Rotar la cámara |
| `Enter` o `Espacio` | Iniciar partida |
| `Tab` | Liberar/capturar el mouse para usar el reproductor |
| `P` | Pausar o reanudar música |
| `N` | Siguiente canción |
| `M` | Silenciar únicamente la música |
| `Escape` | Salir |

Los botones de pausa, siguiente y mute del reproductor también se pueden usar con click después de liberar el cursor con `Tab`.

### Control de PS4

| Entrada | Acción |
| --- | --- |
| Stick izquierdo vertical | Avanzar/retroceder |
| Stick izquierdo horizontal | Desplazamiento lateral |
| Stick derecho horizontal | Rotar la cámara |
| `X` u `Options` | Iniciar partida o volver al menú |

El control puede conectarse o desconectarse durante la ejecución. Tiene zona muerta radial/axial para evitar drift y puede utilizarse al mismo tiempo que el teclado y el mouse.

## Formato del nivel

El mapa está en [`assets/niveles/prueba.txt`](assets/niveles/prueba.txt) y usa una grilla rectangular cerrada:

- `.`: espacio transitable.
- `P`: posición inicial del jugador.
- `G`: meta.
- `1`, `2`, `3`: paredes con materiales distintos.

El nivel actual mide 35 × 15 celdas, incluye pasillos, bifurcaciones, callejones y una sala central. El parser rechaza mapas abiertos, filas irregulares, caracteres inválidos, múltiples inicios o metas y paredes sin textura asignada.

## Música y audio

Al comenzar una partida, las siete canciones de `assets/audio/` se ordenan aleatoriamente una vez y se reproducen en loop mediante `rodio`. El panel muestra canción, tiempo real, duración y progreso. La reproducción corre en un hilo separado del render.

Los pasos y el sonido de éxito usan sinks independientes sobre el mismo mezclador, por lo que no cortan la música. El mute del panel afecta sólo la playlist.

## Objetivos de la rúbrica implementados

| Criterio | Implementación |
| --- | --- |
| Soporte a mando | Control de PS4 mediante `gilrs`, con sticks, hot-plug y deadzone |
| Estética del nivel | Tres materiales cohesivos, zonas cromáticas, textura mapping y luz por distancia/orientación |
| 15 FPS estables | Loop objetivo de 60 FPS y contador visible en pantalla |
| Cámara con movimiento | Avance, retroceso, strafe y rotación con delta-time |
| Rotación con mouse | Movimiento horizontal con sensibilidad configurable y cursor capturado |
| Minimap | Overlay en esquina con nivel, jugador, orientación y meta |
| Música de fondo | Playlist en loop con reproductor interactivo |
| Música de Taylor Swift | Siete canciones declaradas y barajadas al iniciar |
| Efectos de sonido | Pasos por distancia recorrida y efecto de éxito mezclados con la música |
| Pantalla de bienvenida | Menú inicial con instrucciones |
| Pantalla de éxito | Transición al alcanzar `G` y opción de volver al menú |

## Estructura principal

- `src/raycaster.rs`: DDA, proyección, fondo e iluminación.
- `src/player.rs`: movimiento y colisión circular con barrido.
- `src/level.rs`: parser y validación del mapa.
- `src/texture.rs`: carga, sampleo y niveles de iluminación de texturas.
- `src/gamepad_input.rs`: entrada de control y deadzones.
- `src/music.rs`: playlist, hilo de audio y efectos.
- `src/music_ui.rs`: mini reproductor superpuesto.
- `src/minimap.rs`: minimapa y marcador de meta.
