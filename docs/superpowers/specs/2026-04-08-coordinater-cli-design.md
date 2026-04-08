# coordinater — CLI de automação de desktop

## Visão geral

CLI de automação de desktop em Rust, inspirado no PyAutoGUI. Permite controlar mouse, teclado, capturar tela, encontrar imagens na tela e desenhar overlays. Cada chamada executa uma única ação atômica. Todas as ações são logadas.

## Subcomandos

```
coordinater <subcomando> [args] [--monitor <id>]
```

### Flag global

| Flag | Descrição | Default |
|------|-----------|---------|
| `--monitor <id>` | Monitor alvo | Primário |

### Lista de subcomandos

| Comando | Args | Descrição |
|---------|------|-----------|
| `monitors` | — | Lista todos os monitores disponíveis |
| `screenshot` | `-o <file>` | Captura tela do monitor |
| `move` | `<x> <y>` | Move o mouse para x,y |
| `click` | `<x> <y>` | Click esquerdo em x,y |
| `doubleclick` | `<x> <y>` | Duplo click em x,y |
| `rightclick` | `<x> <y>` | Click direito em x,y |
| `drag` | `<x1> <y1> <x2> <y2>` | Arrasta de (x1,y1) até (x2,y2) |
| `scroll` | `<amount>` | Scroll (positivo=cima, negativo=baixo) |
| `key` | `<key>` | Pressiona uma tecla |
| `hotkey` | `<key1> <key2> ...` | Combinação de teclas (ex: `ctrl c`) |
| `type` | `<text>` | Digita uma string inteira |
| `locate` | `<image>` | Busca imagem na tela, retorna coordenadas |
| `draw` | `<shape> [args] --color <cor> --duration <s>` | Desenha overlay na tela |

### Detalhes por subcomando

**monitors:**
```
$ coordinater monitors
[coordinater] monitor 1: 1920x1080 (primary)
[coordinater] monitor 2: 2560x1440
```

**screenshot:**
```
$ coordinater screenshot -o out.png --monitor 2
[coordinater] screenshot saved to out.png from monitor 2 (2560x1440)
```
- `-o` é obrigatório, define o path do arquivo de saída

**move, click, doubleclick, rightclick:**
```
$ coordinater click 200 300
[coordinater] click at x=200,y=300 on monitor 1 (1920x1080)

$ coordinater doubleclick 200 300 --monitor 2
[coordinater] doubleclick at x=200,y=300 on monitor 2 (2560x1440)
```
- Coordenadas são relativas ao monitor selecionado (0,0 = canto superior esquerdo)
- Validadas contra os limites do monitor antes de executar

**drag:**
```
$ coordinater drag 100 100 400 400
[coordinater] drag from x=100,y=100 to x=400,y=400 on monitor 1 (1920x1080)
```
- Mouse down em (x1,y1), move até (x2,y2), mouse up
- Ambos os pontos são validados contra limites

**scroll:**
```
$ coordinater scroll 5
[coordinater] scroll up by 5 on monitor 1 (1920x1080)

$ coordinater scroll -3
[coordinater] scroll down by 3 on monitor 1 (1920x1080)
```

**key:**
```
$ coordinater key enter
[coordinater] key press: enter
```

**hotkey:**
```
$ coordinater hotkey ctrl c
[coordinater] hotkey: ctrl+c
```
- Pressiona todas as teclas na ordem, depois solta na ordem inversa

**type:**
```
$ coordinater type "hello world"
[coordinater] typed: "hello world"
```

**locate:**
```
$ coordinater locate icon.png --monitor 1
[coordinater] locate found icon.png at x=450,y=230 on monitor 1 (1920x1080)
```
- `--threshold <float>` — sensibilidade do matching (default: 0.8)
- Tira screenshot do monitor, faz template matching com a imagem de referência
- Retorna centro da região encontrada
- Exit code != 0 se não encontrar

**draw:**
```
$ coordinater draw line 0 0 500 500 --color red --duration 3
[coordinater] draw line from (0,0) to (500,500) on monitor 1 (1920x1080) [duration: 3s]

$ coordinater draw rect 100 100 300 200 --color blue
[coordinater] draw rect at (100,100) size 300x200 on monitor 2 (2560x1440) [duration: 3s]

$ coordinater draw circle 500 500 50 --color green
[coordinater] draw circle at (500,500) radius 50 on monitor 1 (1920x1080) [duration: 3s]
```
- Shapes suportados: `line`, `rect`, `circle`
- `--color` — cor do desenho (red, green, blue, yellow, white — default: red)
- `--duration` — tempo em segundos que o overlay persiste (default: 3)
- Cria janela transparente, always-on-top, click-through sobre o monitor

## Coordenadas e validação

- Todas as coordenadas são **relativas ao monitor selecionado**
- (0,0) = canto superior esquerdo do monitor
- Antes de executar qualquer ação, as coordenadas são validadas contra a resolução do monitor
- Se fora dos limites, retorna erro sem executar a ação:
  ```
  [coordinater] error: coordinates (2000,500) out of bounds for monitor 1 (1920x1080)
  ```

## Logging

Toda ação loga no `stdout`, sempre, sem opção de silenciar:

```
[coordinater] <ação> <detalhes> on monitor <id> (<largura>x<altura>)
```

Erros vão pro `stderr` com exit code != 0:

```
[coordinater] error: <mensagem>
```

Erros possíveis:
- Coordenadas fora dos limites do monitor
- Monitor não encontrado
- Imagem não encontrada no locate
- Arquivo não encontrado
- Falha ao capturar tela

## Arquitetura

```
src/
  main.rs          -- entry point, chama cli::parse e executa
  cli.rs           -- definição dos subcomandos com clap derive
  monitor.rs       -- detecção de monitores, resolução, validação de limites
  events.rs        -- mouse, teclado, scroll via enigo
  screenshot.rs    -- captura de tela de um monitor específico via xcap
  locate.rs        -- template matching com image/imageproc
  overlay.rs       -- janela transparente com winit + tiny-skia
```

**Fluxo de execução:**

1. `main.rs` faz parse dos args via `cli.rs`
2. Resolve o monitor alvo (default: primário)
3. Valida coordenadas se aplicável
4. Executa a ação no módulo correspondente
5. Loga o resultado no stdout

## Dependências

| Crate | Uso |
|-------|-----|
| `clap` (derive) | CLI e subcomandos |
| `enigo` | Controle de mouse e teclado |
| `xcap` | Captura de tela |
| `image` | Manipulação de imagens |
| `imageproc` | Template matching e primitivas de desenho |
| `winit` | Criação de janela transparente para overlay |
| `tiny-skia` | Renderização de primitivas no overlay |

## Decisões de design

1. **Uma ação por chamada** — filosofia Unix, composável via shell
2. **Coordenadas relativas ao monitor** — mais intuitivo que coordenadas absolutas do desktop
3. **Sempre loga** — transparência total, sem modo silencioso
4. **Validação antes da execução** — nunca executa uma ação com coordenadas inválidas
5. **Sem panic** — todos os erros propagados com Result e mensagens claras
