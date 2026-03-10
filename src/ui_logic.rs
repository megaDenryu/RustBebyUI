use bevy::prelude::*;

// =============================================================================
// コンポーネント (Tags and Data)
// =============================================================================
#[derive(Component)]
pub struct MainText;

#[derive(Component)]
pub struct ClickMeButton;

// =============================================================================
// UI 構築 (View 層)
// =============================================================================
// SengenUI のようなメソッドチェーンを模した構築
pub fn setup_ui(mut commands: Commands) {
    commands.spawn(Camera2d);

    // ルートコンテナの構築
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(Color::BLACK),
        ))
        .with_children(|parent| {
            // メインテキスト
            parent.spawn((
                Text::new("Rust + Bevy UI"),
                TextFont {
                    font_size: 40.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                MainText, // タグ
            ));

            // ボタン
            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(200.0),
                        height: Val::Px(60.0),
                        border: UiRect::all(Val::Px(2.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        margin: UiRect::top(Val::Px(20.0)),
                        ..default()
                    },
                    BorderColor::all(Color::WHITE),
                    BackgroundColor(Color::srgb(0.1, 0.1, 0.1)),
                    ClickMeButton, // タグ
                ))
                .with_child((
                    Text::new("ここをクリック"),
                    TextFont {
                        font_size: 20.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));
        });
}

// =============================================================================
// イベント・ロジック (Orchestrator 層)
// =============================================================================
pub fn handle_button_interaction(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
    mut text_query: Query<&mut Text, With<MainText>>,
) {
    for (interaction, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = BackgroundColor(Color::srgb(0.3, 0.3, 0.3));
                for mut text in &mut text_query {
                    text.0 = "Hello from Bevy!".to_string();
                }
            }
            Interaction::Hovered => {
                *color = BackgroundColor(Color::srgb(0.2, 0.2, 0.2));
            }
            Interaction::None => {
                *color = BackgroundColor(Color::srgb(0.1, 0.1, 0.1));
            }
        }
    }
}
