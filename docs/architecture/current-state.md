# Архитектурный документ: Battle City - Текущее состояние

## Общая информация

**Проект:** Battle City (Танковая битва)  
**Движок:** Bevy 0.17  
**Язык:** Rust (Edition 2021)  
**Архитектурный подход:** Entity Component System (ECS)  
**Дата документа:** 2024

## Описание проекта

Battle City - это ремейк классической игры "Танковая битва" (Battle City), реализованный на игровом движке Bevy. Игра поддерживает одиночный и локальный мультиплеер режимы, систему уровней, физику столкновений и AI противников.

### Основные возможности

- ✅ Редактирование уровней через LDTK
- ✅ Загрузка и переключение уровней
- ✅ Система спавна игроков и врагов
- ✅ Физический движок для коллизий (Rapier2D)
- ✅ Анимации (игроки, враги, взрывы, вода, щиты)
- ✅ Игровой UI
- ✅ Звуковые эффекты
- ✅ Пауза игры
- ✅ AI врагов
- ✅ Локальный мультиплеер (2 игрока)
- ✅ WASM поддержка

## Архитектура проекта

### Структура модулей

Проект организован в виде модульной структуры с четким разделением ответственности:

```
src/
├── main.rs          # Точка входа, регистрация систем и состояний
├── common.rs        # Общие типы, константы, ресурсы
├── player.rs        # Логика игрока
├── enemy.rs         # Логика врагов и AI
├── bullet.rs        # Логика пуль и взрывов
├── level.rs         # Управление уровнями и картой
├── ui.rs            # Пользовательский интерфейс
└── area.rs          # Границы игровой области
```

### Диаграмма зависимостей модулей

```mermaid
graph TD
    A[main.rs] --> B[common.rs]
    A --> C[player.rs]
    A --> D[enemy.rs]
    A --> E[bullet.rs]
    A --> F[level.rs]
    A --> G[ui.rs]
    A --> H[area.rs]
    
    C --> B
    D --> B
    E --> B
    F --> B
    G --> B
    H --> B
    
    C --> F
    D --> F
    E --> C
    E --> D
    E --> F
    E --> H
    
    style A fill:#ff6b6b
    style B fill:#4ecdc4
    style C fill:#95e1d3
    style D fill:#95e1d3
    style E fill:#95e1d3
    style F fill:#95e1d3
    style G fill:#95e1d3
    style H fill:#95e1d3
```

**Легенда:**
- 🔴 **main.rs** - точка входа, регистрация всех систем
- 🔵 **common.rs** - общие типы и ресурсы, используется всеми модулями
- 🟢 **Остальные модули** - специализированные модули с зависимостями

### Принципы архитектуры

1. **ECS подход (Entity Component System)**
   - Использование компонентов Bevy для данных
   - Системы для логики
   - Ресурсы для глобального состояния

2. **Разделение ответственности**
   - Каждый модуль отвечает за свою область
   - Минимальная связанность между модулями

3. **Event-driven архитектура**
   - Использование событий Bevy для коммуникации между системами
   - `ExplosionEvent`, `SpawnPlayerEvent`, `HomeDyingEvent`

### ECS Архитектура

```mermaid
graph TB
    subgraph "Entities (Сущности)"
        E1[Player Entity]
        E2[Enemy Entity]
        E3[Bullet Entity]
        E4[Level Item Entity]
    end
    
    subgraph "Components (Компоненты)"
        C1[PlayerNo<br/>Direction<br/>Transform<br/>Velocity]
        C2[Enemy<br/>Direction<br/>Transform]
        C3[Bullet<br/>Direction<br/>Transform]
        C4[LevelItem<br/>Transform]
    end
    
    subgraph "Systems (Системы)"
        S1[players_move<br/>players_attack<br/>animate_players]
        S2[enemies_move<br/>enemies_attack<br/>animate_enemies]
        S3[move_bullet<br/>handle_bullet_collision]
        S4[animate_water<br/>auto_switch_level]
    end
    
    subgraph "Resources (Ресурсы)"
        R1[PlayerLives<br/>MultiplayerMode]
        R2[LevelSpawnedEnemies<br/>LevelSelection]
        R3[GameSounds<br/>ExplosionAssets]
        R4[Time<br/>AssetServer]
    end
    
    subgraph "Events (События)"
        EV1[SpawnPlayerEvent]
        EV2[ExplosionEvent]
        EV3[HomeDyingEvent]
        EV4[CollisionEvent]
    end
    
    E1 --> C1
    E2 --> C2
    E3 --> C3
    E4 --> C4
    
    C1 --> S1
    C2 --> S2
    C3 --> S3
    C4 --> S4
    
    S1 --> R1
    S2 --> R2
    S3 --> R3
    S4 --> R4
    
    S1 -.->|emits| EV1
    S3 -.->|emits| EV2
    S3 -.->|emits| EV3
    S3 -.->|reads| EV4
    
    style E1 fill:#ff6b6b
    style E2 fill:#ff6b6b
    style E3 fill:#ff6b6b
    style E4 fill:#ff6b6b
    style C1 fill:#4ecdc4
    style C2 fill:#4ecdc4
    style C3 fill:#4ecdc4
    style C4 fill:#4ecdc4
    style S1 fill:#95e1d3
    style S2 fill:#95e1d3
    style S3 fill:#95e1d3
    style S4 fill:#95e1d3
    style R1 fill:#ffe66d
    style R2 fill:#ffe66d
    style R3 fill:#ffe66d
    style R4 fill:#ffe66d
    style EV1 fill:#a8e6cf
    style EV2 fill:#a8e6cf
    style EV3 fill:#a8e6cf
    style EV4 fill:#a8e6cf
```

**Описание:**
- **Entities** - игровые объекты (танки, пули, элементы уровня)
- **Components** - данные, прикрепленные к сущностям
- **Systems** - логика, обрабатывающая компоненты
- **Resources** - глобальное состояние игры
- **Events** - асинхронная коммуникация между системами

## Основные компоненты

### Структура компонентов игровых сущностей

```mermaid
erDiagram
    PLAYER ||--o{ SHIELD : "has"
    PLAYER ||--o{ BULLET : "shoots"
    ENEMY ||--o{ BULLET : "shoots"
    BULLET ||--o| EXPLOSION : "creates"
    LEVEL_ITEM ||--o{ BULLET : "collides"
    PLAYER ||--o{ LEVEL_ITEM : "interacts"
    ENEMY ||--o{ LEVEL_ITEM : "interacts"
    
    PLAYER {
        PlayerNo player_no
        Transform transform
        Velocity velocity
        Direction direction
        Sprite sprite
        AnimationTimer anim_timer
        AnimationIndices anim_indices
        TankRefreshBulletTimer bullet_timer
        RigidBody rigid_body
        Collider collider
    }
    
    ENEMY {
        Enemy marker
        Transform transform
        Direction direction
        Sprite sprite
        AnimationTimer anim_timer
        AnimationIndices anim_indices
        TankRefreshBulletTimer bullet_timer
        EnemyChangeDirectionTimer direction_timer
        RigidBody rigid_body
        Collider collider
    }
    
    BULLET {
        Bullet bullet_type
        Direction direction
        Transform transform
        Sprite sprite
        Collider collider
        RigidBody rigid_body
        Sensor sensor
    }
    
    SHIELD {
        Shield marker
        Sprite sprite
        AnimationTimer anim_timer
        AnimationIndices anim_indices
        ShieldRemoveTimer remove_timer
    }
    
    EXPLOSION {
        Explosion marker
        Sprite sprite
        AnimationTimer anim_timer
        AnimationIndices anim_indices
    }
    
    LEVEL_ITEM {
        LevelItem item_type
        Transform transform
        Sprite sprite
        RigidBody rigid_body
        Collider collider
    }
```

### 1. Common (`common.rs`)

**Константы:**
- `LEVEL_ROWS: i32 = 18` - количество строк в уровне
- `LEVEL_COLUMNS: i32 = 27` - количество столбцов
- `TILE_SIZE: f32 = 32.0` - размер тайла
- `MAX_LEVELS: i32 = 2` - максимальное количество уровней
- `MAX_LIVE_ENEMIES: i32 = 5` - максимальное количество врагов одновременно
- `ENEMIES_PER_LEVEL: i32 = 12` - врагов на уровень
- `PLAYER_SPEED: f32 = 150.0` - скорость игрока
- `ENEMY_SPEED: f32 = 100.0` - скорость врага

**Состояния приложения (`AppState`):**
```rust
pub enum AppState {
    StartMenu,    // Главное меню
    Playing,      // Игровой процесс
    Paused,       // Пауза
    GameOver,     // Конец игры
}
```

**Режимы игры (`MultiplayerMode`):**
```rust
pub enum MultiplayerMode {
    SinglePlayer,  // Одиночная игра
    TwoPlayers,    // Два игрока
}
```

**Направление (`Direction`):**
```rust
pub enum Direction {
    Left, Right, Up, Down,
}
```

**Компоненты:**
- `AnimationTimer` - таймер для анимаций
- `AnimationIndices` - индексы кадров анимации
- `TankRefreshBulletTimer` - таймер перезарядки выстрела

**Ресурсы:**
- `GameSounds` - звуковые эффекты игры
- `ScheduledDespawn` - дедупликация запросов на удаление сущностей
- `PlayerLives` - жизни игроков

### 2. Player (`player.rs`)

**Компоненты:**
- `PlayerNo(u32)` - номер игрока (1 или 2)
- `Shield` - щит защиты при спавне
- `ShieldRemoveTimer` - таймер удаления щита
- `Born` - эффект появления
- `BornRemoveTimer` - таймер удаления эффекта появления

**События:**
- `SpawnPlayerEvent` - событие спавна игрока

**Ресурсы:**
- `PlayerLives` - жизни игроков (player1, player2)

**Основные системы:**
- `auto_spawn_players` - автоматический спавн игроков
- `players_move` - управление движением (WASD для P1, стрелки для P2)
- `players_attack` - стрельба (Space для P1, Enter для P2)
- `animate_players` - анимация движения танков
- `animate_shield` - анимация щита
- `remove_shield` - удаление щита после таймера
- `animate_born` - анимация появления
- `spawn_born` - создание эффекта появления

**Особенности:**
- Поддержка двух игроков с разными управлениями
- Система жизней
- Защитный щит на 5 секунд после спавна
- Анимация появления перед спавном танка

#### Диаграмма компонентов Player

```mermaid
classDiagram
    class PlayerEntity {
        +PlayerNo player_no
        +Transform transform
        +Velocity velocity
        +Direction direction
        +Sprite sprite
        +AnimationTimer anim_timer
        +AnimationIndices anim_indices
        +TankRefreshBulletTimer bullet_timer
        +RigidBody rigid_body
        +Collider collider
    }
    
    class ShieldEntity {
        +Shield marker
        +Sprite sprite
        +AnimationTimer anim_timer
        +AnimationIndices anim_indices
        +ShieldRemoveTimer remove_timer
    }
    
    class BornEntity {
        +Born marker
        +PlayerNo player_no
        +Sprite sprite
        +AnimationTimer anim_timer
        +AnimationIndices anim_indices
        +BornRemoveTimer remove_timer
    }
    
    class PlayerLives {
        +i8 player1
        +i8 player2
    }
    
    class SpawnPlayerEvent {
        +Vec2 pos
        +PlayerNo player_no
    }
    
    PlayerEntity "1" *-- "0..1" ShieldEntity : has shield
    PlayerEntity "1" --> "1" PlayerLives : uses
    BornEntity "1" --> "1" SpawnPlayerEvent : emits
    SpawnPlayerEvent "1" --> "1" PlayerEntity : creates
```

### 3. Enemy (`enemy.rs`)

**Компоненты:**
- `Enemy` - маркер врага
- `EnemyChangeDirectionTimer` - таймер смены направления

**Ресурсы:**
- `LevelSpawnedEnemies` - количество заспавненных врагов на уровне

**Основные системы:**
- `auto_spawn_enemies` - автоматический спавн врагов
- `spawn_enemy` - создание врага
- `enemies_move` - AI движение врагов
- `enemies_attack` - автоматическая стрельба врагов
- `handle_enemy_collision` - обработка коллизий врагов
- `animate_enemies` - анимация врагов

**AI логика:**
- Случайное движение с весами (вниз предпочтительнее)
- Проверка препятствий перед движением
- Смена направления при коллизии
- Автоматическая стрельба с интервалом

**Особенности:**
- Максимум 5 врагов одновременно на поле
- Спавн на случайных позициях из маркеров
- Проверка расстояния от других танков при спавне
- 8 различных визуальных типов врагов

### 4. Bullet (`bullet.rs`)

**Компоненты:**
- `Bullet` (enum) - тип пули (Player/Enemy)
- `Explosion` - маркер взрыва

**События:**
- `ExplosionEvent` - событие взрыва с типом (BigExplosion/BulletExplosion)

**Ресурсы:**
- `ExplosionAssets` - ресурсы для анимаций взрывов

**Основные системы:**
- `spawn_bullet` - создание пули
- `move_bullet` - движение пули
- `handle_bullet_collision` - обработка коллизий пуль
- `spawn_explosion` - создание взрыва
- `animate_explosion` - анимация взрыва

**Логика коллизий:**
- Пули игрока уничтожают врагов и каменные стены
- Пули врагов уничтожают игроков (если нет щита)
- Пули не проходят через железные стены
- Пули уничтожаются при попадании в границы карты
- Попадание в дом вызывает Game Over

**Особенности:**
- Разные типы взрывов (большой/маленький)
- Звуковые эффекты для взрывов
- Анимации взрывов с разным количеством кадров

#### Диаграмма обработки коллизий пуль

```mermaid
flowchart TD
    Start[Пуля движется] --> Check{Коллизия?}
    
    Check -->|Нет| Move[move_bullet<br/>Продолжает движение]
    Move --> Check
    
    Check -->|Да| Collision[CollisionEvent]
    
    Collision --> Type{Тип пули?}
    
    Type -->|Player| PlayerBullet[Пуля игрока]
    Type -->|Enemy| EnemyBullet[Пуля врага]
    
    PlayerBullet --> Target1{Цель?}
    Target1 -->|Enemy| DestroyEnemy[Уничтожить врага<br/>BigExplosion]
    Target1 -->|StoneWall| DestroyWall[Уничтожить стену<br/>BulletExplosion]
    Target1 -->|IronWall| BounceIron[Отскок от стены<br/>BulletExplosion]
    Target1 -->|Home| GameOver[Game Over<br/>BigExplosion]
    Target1 -->|AreaWall| DestroyBullet[Уничтожить пулю<br/>BulletExplosion]
    
    EnemyBullet --> Target2{Цель?}
    Target2 -->|Player| CheckShield{Есть щит?}
    CheckShield -->|Да| ShieldBlock[Щит блокирует<br/>BulletExplosion]
    CheckShield -->|Нет| DestroyPlayer[Уничтожить игрока<br/>BigExplosion<br/>Проверка жизней]
    Target2 -->|StoneWall| DestroyWall
    Target2 -->|IronWall| BounceIron
    Target2 -->|Home| GameOver
    Target2 -->|AreaWall| DestroyBullet
    
    DestroyEnemy --> Explosion[spawn_explosion]
    DestroyWall --> Explosion
    BounceIron --> Explosion
    GameOver --> Explosion
    DestroyBullet --> Explosion
    ShieldBlock --> Explosion
    DestroyPlayer --> Explosion
    
    Explosion --> End[Конец обработки]
    
    style Start fill:#95e1d3
    style Collision fill:#ff6b6b
    style Explosion fill:#ffe66d
    style GameOver fill:#ff4757
```

### 5. Level (`level.rs`)

**Компоненты:**
- `LevelItem` (enum) - элементы уровня:
  - `StoneWall` - каменная стена (разрушаемая)
  - `IronWall` - железная стена (неразрушаемая)
  - `Tree` - дерево (декоративное)
  - `Water` - вода (с анимацией)
  - `Home` - база (цель защиты)
- `Player1Marker` - маркер позиции игрока 1
- `Player2Marker` - маркер позиции игрока 2
- `EnemiesMarker` - маркер позиции врагов

**Bundles для LDTK:**
- `StoneWallBundle`
- `IronWallBundle`
- `TreeBundle`
- `WaterBundle`
- `HomeBundle`
- `Player1MarkerBundle`
- `Player2MarkerBundle`
- `EnemiesMarkerBundle`

**Основные системы:**
- `setup_levels` - загрузка LDTK проекта
- `spawn_ldtk_entity` - спавн сущностей из LDTK
- `animate_water` - анимация воды
- `auto_switch_level` - автоматическое переключение уровней
- `animate_home` - анимация разрушения базы

**Особенности:**
- Интеграция с LDTK для редактирования уровней
- Автоматическое создание коллайдеров для элементов
- Анимация воды
- Переключение между уровнями при уничтожении всех врагов

### 6. UI (`ui.rs`)

**Компоненты:**
- `OnStartMenuScreen` - маркер главного меню
- `OnStartMenuScreenMultiplayerModeFlag` - флаг режима игры
- `OnGameOverScreen` - маркер экрана Game Over

**Основные системы:**
- `setup_start_menu` - создание главного меню
- `setup_game_over` - создание экрана Game Over
- `start_game` - запуск игры (Enter/Space)
- `switch_multiplayer_mode` - переключение режима (↑/↓)
- `pause_game` - пауза (Escape)
- `unpause_game` - снятие с паузы (Escape)
- `animate_game_over` - анимация Game Over экрана
- `despawn_screen` - очистка экранов

**Особенности:**
- Главное меню с выбором режима игры
- Анимированный экран Game Over
- Система паузы с защитой от двойного срабатывания

### 7. Area (`area.rs`)

**Компоненты:**
- `AreaWall` - маркер граничной стены

**Основные системы:**
- `setup_wall` - создание граничных стен игровой области

**Особенности:**
- Невидимые стены по границам уровня
- Физические коллайдеры для предотвращения выхода за границы

## Физика и коллизии

### Физический движок: Rapier2D

**Конфигурация:**
- `pixels_per_meter: 100.0`
- Гравитация отключена (`gravity = Vec2::ZERO`)

**Типы тел:**
- `RigidBody::Dynamic` - для танков и пуль
- `RigidBody::Fixed` - для стен и элементов уровня

**Коллайдеры:**
- Танки: `Collider::ball()` - круглые коллайдеры
- Пули: `Collider::cuboid(2.0, 2.0)` - маленькие квадратные
- Стены: `Collider::cuboid()` - прямоугольные
- Пули используют `Sensor` для обнаружения коллизий без физического взаимодействия

**События коллизий:**
- `CollisionEvent::Started` - начало коллизии
- `CollisionEvent::Stopped` - конец коллизии
- Используется `ActiveEvents::COLLISION_EVENTS` для получения событий

### Диаграмма физических объектов

```mermaid
graph TB
    subgraph "Dynamic Bodies (Динамические тела)"
        P[Player Tank<br/>RigidBody::Dynamic<br/>Collider::ball<br/>Velocity]
        E[Enemy Tank<br/>RigidBody::Dynamic<br/>Collider::cuboid<br/>Transform]
        B[Bullet<br/>RigidBody::Dynamic<br/>Collider::cuboid<br/>Sensor]
    end
    
    subgraph "Fixed Bodies (Статичные тела)"
        SW[StoneWall<br/>RigidBody::Fixed<br/>Collider::cuboid]
        IW[IronWall<br/>RigidBody::Fixed<br/>Collider::cuboid]
        W[Water<br/>RigidBody::Fixed<br/>Collider::cuboid]
        H[Home<br/>RigidBody::Fixed<br/>Collider::cuboid]
        AW[AreaWall<br/>RigidBody::Fixed<br/>Collider::cuboid]
    end
    
    subgraph "Collision Detection"
        CD[CollisionEvent<br/>Started/Stopped]
    end
    
    P -->|может столкнуться| SW
    P -->|может столкнуться| IW
    P -->|может столкнуться| W
    P -->|может столкнуться| AW
    P -->|может столкнуться| E
    
    E -->|может столкнуться| SW
    E -->|может столкнуться| IW
    E -->|может столкнуться| W
    E -->|может столкнуться| AW
    
    B -->|обнаруживает| P
    B -->|обнаруживает| E
    B -->|обнаруживает| SW
    B -->|обнаруживает| IW
    B -->|обнаруживает| H
    B -->|обнаруживает| AW
    
    P -.->|генерирует| CD
    E -.->|генерирует| CD
    B -.->|генерирует| CD
    
    CD -.->|обрабатывает| handle_bullet_collision
    CD -.->|обрабатывает| handle_enemy_collision
    
    style P fill:#4ecdc4
    style E fill:#ff6b6b
    style B fill:#ffe66d
    style SW fill:#95e1d3
    style IW fill:#95e1d3
    style W fill:#95e1d3
    style H fill:#ff4757
    style AW fill:#a8e6cf
    style CD fill:#ffd93d
```

## Управление состоянием

### State Machine

Игра использует систему состояний Bevy (`AppState`):

```mermaid
stateDiagram-v2
    [*] --> StartMenu: Запуск приложения
    
    StartMenu --> Playing: Enter/Space<br/>(start_game)
    StartMenu --> StartMenu: ↑/↓<br/>(switch_multiplayer_mode)
    
    Playing --> Paused: Escape<br/>(pause_game)
    Playing --> GameOver: Уничтожение базы<br/>или конец жизней
    
    Paused --> Playing: Escape<br/>(unpause_game)
    
    GameOver --> StartMenu: Автоматически<br/>через 1 секунду<br/>(animate_game_over)
    
    note right of StartMenu
        OnEnter:
        - cleanup_level_items
        - cleanup_players
        - reset_player_lives
        - setup_start_menu
    end note
    
    note right of Playing
        OnEnter:
        - setup_levels
        
        Update:
        - players_move
        - enemies_move
        - move_bullet
        - handle_bullet_collision
    end note
    
    note right of Paused
        Update:
        - unpause_game
        - animate_players (только анимация)
    end note
    
    note right of GameOver
        OnEnter:
        - setup_game_over
        
        OnExit:
        - despawn_screen
    end note
```

**Переходы:**
- `StartMenu → Playing`: Enter/Space в главном меню
- `Playing → Paused`: Escape
- `Paused → Playing`: Escape
- `Playing → GameOver`: Уничтожение базы или конец жизней
- `GameOver → StartMenu`: Автоматически через 1 секунду

### Ресурсы состояния

- `LevelSelection` - текущий выбранный уровень
- `LevelSpawnedEnemies` - количество заспавненных врагов
- `PlayerLives` - жизни игроков
- `MultiplayerMode` - режим игры

## Системы жизненного цикла

### Startup системы
- `setup_camera` - настройка камеры
- `setup_rapier` - настройка физики
- `setup_wall` - создание границ
- `setup_explosion_assets` - загрузка ресурсов взрывов
- `setup_game_sounds` - загрузка звуков

### OnEnter системы
- `OnEnter(AppState::StartMenu)`: очистка и сброс состояния
- `OnEnter(AppState::Playing)`: загрузка уровня
- `OnEnter(AppState::GameOver)`: показ экрана Game Over

### OnExit системы
- `OnExit(AppState::StartMenu)`: удаление UI меню
- `OnExit(AppState::GameOver)`: удаление UI Game Over

### Update системы
Системы обновления организованы по состояниям с использованием `run_if(in_state(AppState::...))`.

#### Диаграмма игрового цикла (Playing State)

```mermaid
sequenceDiagram
    participant Input as Input System
    participant Player as Player Systems
    participant Enemy as Enemy Systems
    participant Bullet as Bullet Systems
    participant Level as Level Systems
    participant Physics as Rapier2D
    participant Events as Event System
    
    loop Каждый кадр (Update)
        Input->>Player: Keyboard Input
        Player->>Player: players_move
        Player->>Player: players_attack
        Player->>Player: animate_players
        
        Enemy->>Enemy: enemies_move (AI)
        Enemy->>Enemy: enemies_attack
        Enemy->>Enemy: animate_enemies
        Enemy->>Enemy: auto_spawn_enemies
        
        Bullet->>Bullet: move_bullet
        
        Physics->>Events: CollisionEvent
        
        Events->>Bullet: handle_bullet_collision
        Bullet->>Events: ExplosionEvent
        Bullet->>Events: HomeDyingEvent
        
        Events->>Bullet: spawn_explosion
        Events->>Player: SpawnPlayerEvent (if needed)
        
        Level->>Level: animate_water
        Level->>Level: auto_switch_level
        
        Player->>Player: animate_shield
        Player->>Player: remove_shield
        Player->>Player: animate_born
        
        Bullet->>Bullet: animate_explosion
    end
```

#### Диаграмма жизненного цикла сущностей

```mermaid
stateDiagram-v2
    [*] --> Spawn: Создание сущности
    
    state Player {
        [*] --> BornAnimation: spawn_born
        BornAnimation --> Spawned: Через 2 сек
        Spawned --> WithShield: Создан с щитом
        WithShield --> WithoutShield: Через 5 сек
        WithoutShield --> Alive: Играет
        Alive --> Dying: Попадание пули
        Dying --> [*]: Уничтожен
    }
    
    state Enemy {
        [*] --> Spawned: auto_spawn_enemies
        Spawned --> Moving: AI движение
        Moving --> Attacking: Стрельба
        Attacking --> Moving: Продолжает движение
        Moving --> Dying: Попадание пули
        Dying --> [*]: Уничтожен
    }
    
    state Bullet {
        [*] --> Moving: spawn_bullet
        Moving --> Colliding: handle_bullet_collision
        Colliding --> Exploding: ExplosionEvent
        Exploding --> [*]: Уничтожена
    }
    
    Spawn --> Player
    Spawn --> Enemy
    Spawn --> Bullet
```

## Зависимости

### Основные зависимости

```toml
bevy = "0.17"                    # Игровой движок
bevy_rapier2d = { git = "..." }  # Физический движок
bevy_ecs_ldtk = "0.13"           # Интеграция с LDTK
rand = "0.8.5"                   # Генерация случайных чисел
```

### Особенности зависимостей

- **bevy_rapier2d**: Используется форк с поддержкой Bevy 0.17
- **bevy_ecs_ldtk**: Для загрузки уровней из LDTK файлов
- **rand**: Для AI врагов и случайного спавна

## Анимации

### Система анимаций

Все анимации используют общий паттерн:
- `AnimationTimer` - таймер обновления кадров
- `AnimationIndices` - диапазон кадров анимации
- `TextureAtlas` - спрайт-лист с кадрами

### Анимированные объекты

1. **Танки (игроки и враги)**
   - 2 кадра на направление
   - Обновление каждые 0.2 секунды

2. **Щит**
   - 2 кадра
   - Обновление каждые 0.2 секунды

3. **Вода**
   - 2 кадра (индексы 3-4)
   - Обновление каждые 0.2 секунды

4. **Взрывы**
   - Большой взрыв: 5 кадров, 0.05 сек на кадр
   - Маленький взрыв: 3 кадра, 0.05 сек на кадр

5. **Появление**
   - 4 кадра
   - Длительность 2 секунды

## Звуковая система

### Звуковые эффекты

Ресурс `GameSounds` содержит:
- `mode_switch` - переключение режима
- `bullet_explosion` - взрыв пули
- `big_explosion` - большой взрыв
- `player_fire` - выстрел игрока
- `game_over` - конец игры
- `game_pause` - пауза

### Воспроизведение

Звуки воспроизводятся через `AudioPlayer` с `PlaybackSettings::DESPAWN` для автоматического удаления после проигрывания.

## Управление памятью

### ScheduledDespawn

Ресурс `ScheduledDespawn` используется для предотвращения множественного удаления одной сущности в одном кадре. Это важно при обработке событий коллизий, которые могут срабатывать несколько раз.

### Cleanup системы

Каждое состояние имеет системы очистки:
- `cleanup_players`
- `cleanup_enemies`
- `cleanup_bullets`
- `cleanup_explosions`
- `cleanup_level_items`
- `cleanup_ldtk_world`
- `cleanup_born`

## Известные проблемы и TODO

Из кода видно следующие TODO:

1. **TODO в main.rs:**
   - "Танк при столкновении вынужден двигаться" - Танки при столкновении вынуждены двигаться

2. **TODO в enemy.rs:**
   - "Активная атака после обнаружения игрока" - Враги должны активно атаковать при обнаружении игрока
   - "Деревья могут обеспечивать укрытие" - Деревья должны обеспечивать укрытие

3. **TODO в level.rs:**
   - "Игровая победа" - Реализация экрана победы

## Рекомендации по улучшению архитектуры

### 1. Разделение на слои (Clean Architecture)

**Текущее состояние:** Логика смешана с представлением

**Рекомендации:**
- Выделить Domain слой (бизнес-логика)
- Выделить Application слой (оркестрация систем)
- Выделить Infrastructure слой (Bevy, Rapier, LDTK)

### 2. CQRS подход

**Текущее состояние:** Команды и запросы не разделены

**Рекомендации:**
- Разделить системы на Commands (изменение состояния) и Queries (чтение)
- Использовать события для коммуникации между слоями

### 3. DDD подход

**Текущее состояние:** Нет явных агрегатов и доменных моделей

**Рекомендации:**
- Выделить агрегаты: `Player`, `Enemy`, `Level`, `Bullet`
- Инкапсулировать бизнес-логику в компонентах
- Использовать Value Objects для направлений, позиций

### 4. Тестирование (TDD)

**Текущее состояние:** Нет тестов

**Рекомендации:**
- Добавить unit тесты для бизнес-логики
- Добавить integration тесты для систем
- Использовать mock для Bevy систем

### 5. Улучшение модульности

**Рекомендации:**
- Выделить общие системы анимаций в отдельный модуль
- Создать модуль для управления ресурсами
- Разделить UI на подмодули (меню, HUD, экраны)

### 6. Конфигурация

**Рекомендации:**
- Вынести константы в конфигурационные файлы
- Использовать ресурсы для настроек игры
- Поддержка загрузки конфигурации из файлов

## Общая архитектурная схема

### Диаграмма взаимодействия всех систем

```mermaid
graph TB
    subgraph "Input Layer"
        KB[Keyboard Input]
    end
    
    subgraph "Game State Management"
        SM[State Machine<br/>AppState]
    end
    
    subgraph "Player Module"
        PM[players_move]
        PA[players_attack]
        AP[animate_players]
        ASP[auto_spawn_players]
    end
    
    subgraph "Enemy Module"
        EM[enemies_move]
        EA[enemies_attack]
        AE[animate_enemies]
        ASE[auto_spawn_enemies]
    end
    
    subgraph "Bullet Module"
        MB[move_bullet]
        HBC[handle_bullet_collision]
        SE[spawn_explosion]
        AE2[animate_explosion]
    end
    
    subgraph "Level Module"
        SL[setup_levels]
        AL[auto_switch_level]
        AW2[animate_water]
    end
    
    subgraph "UI Module"
        SSM[setup_start_menu]
        SGO[setup_game_over]
        PG[pause_game]
    end
    
    subgraph "Physics Engine"
        RAP[Rapier2D]
        CE[CollisionEvent]
    end
    
    subgraph "Event System"
        EV[Events<br/>ExplosionEvent<br/>SpawnPlayerEvent<br/>HomeDyingEvent]
    end
    
    KB --> PM
    KB --> PA
    KB --> PG
    
    SM --> PM
    SM --> EM
    SM --> MB
    SM --> SSM
    SM --> SGO
    
    PM --> RAP
    EM --> RAP
    MB --> RAP
    
    RAP --> CE
    CE --> HBC
    
    HBC --> EV
    EV --> SE
    EV --> ASP
    EV --> SM
    
    PA --> MB
    EA --> MB
    
    ASP --> PM
    ASE --> EM
    
    AL --> SM
    
    style KB fill:#ff6b6b
    style SM fill:#4ecdc4
    style RAP fill:#95e1d3
    style EV fill:#ffe66d
    style CE fill:#ffd93d
```

### Диаграмма потоков данных

```mermaid
flowchart LR
    subgraph "Входные данные"
        I1[Keyboard Input]
        I2[Time Delta]
        I3[LDTK Level Data]
    end
    
    subgraph "Обработка"
        P1[Player Systems]
        P2[Enemy Systems]
        P3[Bullet Systems]
        P4[Level Systems]
        P5[UI Systems]
    end
    
    subgraph "Физика"
        PH1[Rapier2D<br/>Physics Engine]
        PH2[Collision Detection]
    end
    
    subgraph "События"
        E1[Event Bus]
        E2[ExplosionEvent]
        E3[SpawnPlayerEvent]
        E4[HomeDyingEvent]
    end
    
    subgraph "Выходные данные"
        O1[Transform Updates]
        O2[Animation Frames]
        O3[Sound Effects]
        O4[State Changes]
    end
    
    I1 --> P1
    I1 --> P5
    I2 --> P1
    I2 --> P2
    I2 --> P3
    I2 --> P4
    
    I3 --> P4
    
    P1 --> PH1
    P2 --> PH1
    P3 --> PH1
    
    PH1 --> PH2
    PH2 --> P3
    
    P1 --> E1
    P2 --> E1
    P3 --> E1
    P4 --> E1
    
    E1 --> E2
    E1 --> E3
    E1 --> E4
    
    E2 --> P3
    E3 --> P1
    E4 --> P4
    E4 --> P5
    
    P1 --> O1
    P1 --> O2
    P2 --> O1
    P2 --> O2
    P3 --> O1
    P3 --> O2
    P4 --> O1
    P4 --> O2
    P5 --> O4
    
    E2 --> O3
    E3 --> O1
    E4 --> O4
    
    style I1 fill:#ff6b6b
    style I2 fill:#ff6b6b
    style I3 fill:#ff6b6b
    style PH1 fill:#4ecdc4
    style PH2 fill:#4ecdc4
    style E1 fill:#ffe66d
    style O1 fill:#95e1d3
    style O2 fill:#95e1d3
    style O3 fill:#95e1d3
    style O4 fill:#95e1d3
```

## Заключение

Проект Battle City демонстрирует хорошее понимание ECS архитектуры Bevy и правильное использование систем, компонентов и ресурсов. Код организован модульно, но есть возможности для улучшения в плане применения принципов Clean Architecture, DDD и CQRS.

Основные сильные стороны:
- ✅ Четкое разделение модулей
- ✅ Использование ECS паттернов
- ✅ Event-driven коммуникация
- ✅ Модульная структура

Области для улучшения:
- ⚠️ Применение Clean Architecture принципов
- ⚠️ Разделение на слои (Domain/Application/Infrastructure)
- ⚠️ Добавление тестов
- ⚠️ Выделение бизнес-логики из систем

---

**Версия документа:** 1.1  
**Последнее обновление:** 2024  
**Добавлено:** Архитектурные диаграммы (Mermaid)

