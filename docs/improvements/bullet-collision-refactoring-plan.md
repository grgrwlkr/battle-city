# План рефакторинга системы детекта столкновений пуль

## Текущая проблема
Текущая система `handle_bullet_collision` слишком сложная и содержит логическую ошибку, из-за которой столкновения пуль с игроком не работают корректно.

## Цель
Упростить систему детекта столкновений, разделив её на 3 независимые системы:
1. **Детект столкновения пуль с врагами** (`handle_bullet_enemy_collision`)
2. **Детект столкновения пуль с игроком** (`handle_bullet_player_collision`)
3. **Детект столкновения пуль с другими объектами** (`handle_bullet_other_collision`)

## План реализации

### 1. Система детекта столкновения пуль с врагами
**Функция:** `handle_bullet_enemy_collision`

**Логика:**
- Обрабатывает только события столкновений, где одна сущность - пуля игрока, а другая - враг
- Проверяет: `bullet == Bullet::Player && q_enemies.contains(other_entity)`
- При столкновении:
  - Уничтожает пулю
  - Уничтожает врага
  - Создаёт взрыв (BigExplosion)

**Параметры:**
- `q_bullets: Query<(Entity, &Bullet, &Transform)>`
- `q_enemies: Query<&Transform, With<Enemy>>`
- `mut collision_er: MessageReader<CollisionEvent>`
- `mut explosion_ew: MessageWriter<ExplosionEvent>`
- `mut scheduled: ResMut<ScheduledDespawn>`
- `mut commands: Commands`

### 2. Система детекта столкновения пуль с игроком
**Функция:** `handle_bullet_player_collision`

**Логика:**
- Обрабатывает только события столкновений, где одна сущность - вражеская пуля, а другая - игрок или его щит
- Проверяет: `bullet == Bullet::Enemy`
- Ищет игрока, у которого `other_entity` является либо самим игроком, либо дочерним элементом (щитом)
- При столкновении:
  - Если есть щит: создаёт взрыв (BulletExplosion), пуля уничтожается
  - Если нет щита: уменьшает жизни игрока, уничтожает игрока и пулю, создаёт взрыв (BigExplosion), проверяет Game Over

**Параметры:**
- `q_bullets: Query<(Entity, &Bullet, &Transform)>`
- `q_players: Query<(Entity, &PlayerNo, &Transform, &Children), With<PlayerNo>>`
- `q_shields: Query<Entity, With<Shield>>`
- `mut collision_er: MessageReader<CollisionEvent>`
- `mut explosion_ew: MessageWriter<ExplosionEvent>`
- `mut player_lives: ResMut<PlayerLives>`
- `multiplayer_mode: Res<MultiplayerMode>`
- `mut app_state: ResMut<NextState<AppState>>`
- `mut scheduled: ResMut<ScheduledDespawn>`
- `mut commands: Commands`

### 3. Система детекта столкновения пуль с другими объектами
**Функция:** `handle_bullet_other_collision`

**Логика:**
- Обрабатывает столкновения пуль с:
  - Level Items (стены, база, деревья, вода)
  - Area Walls (границы карты)
- При столкновении:
  - StoneWall: уничтожает пулю и стену, создаёт взрыв
  - Tree: уничтожает пулю, создаёт взрыв
  - IronWall: уничтожает пулю, создаёт взрыв
  - Home: уничтожает пулю, создаёт взрыв, отправляет HomeDyingEvent
  - AreaWall: уничтожает пулю, создаёт взрыв

**Параметры:**
- `q_bullets: Query<(Entity, &Bullet, &Transform)>`
- `q_level_items: Query<(&LevelItem, &GlobalTransform, &mut Sprite)>`
- `q_area_wall: Query<(), With<AreaWall>>`
- `mut collision_er: MessageReader<CollisionEvent>`
- `mut explosion_ew: MessageWriter<ExplosionEvent>`
- `mut home_dying_ew: MessageWriter<HomeDyingEvent>`
- `mut scheduled: ResMut<ScheduledDespawn>`
- `mut commands: Commands`

## Порядок выполнения систем

Системы должны выполняться в следующем порядке:
1. `handle_bullet_enemy_collision` - сначала обрабатываем столкновения с врагами
2. `handle_bullet_player_collision` - затем столкновения с игроком
3. `handle_bullet_other_collision` - в конце столкновения с другими объектами

Это важно, чтобы приоритетные столкновения (игрок, враг) обрабатывались раньше, чем столкновения с объектами уровня.

## Общая логика для всех систем

Каждая система должна:
1. Читать события из `collision_er`
2. Определять, какая сущность - пуля, а какая - другой объект
3. Проверять, что пуля существует в запросе
4. Обрабатывать только релевантные столкновения (игнорировать остальные)
5. Использовать `scheduled` для предотвращения двойного уничтожения

## Преимущества нового подхода

1. **Простота**: каждая система отвечает только за один тип столкновений
2. **Читаемость**: легче понять логику каждой системы
3. **Отладка**: проще найти и исправить ошибки
4. **Тестируемость**: можно тестировать каждую систему отдельно
5. **Расширяемость**: легко добавить новые типы столкновений

## Шаги реализации

1. Создать функцию `handle_bullet_enemy_collision`
2. Создать функцию `handle_bullet_player_collision`
3. Создать функцию `handle_bullet_other_collision`
4. Удалить старую функцию `handle_bullet_collision`
5. Обновить регистрацию систем в `BulletPlugin`
6. Протестировать каждую систему отдельно
7. Удалить временные логи отладки
