use bevy::prelude::*;

use crate::GameAssets;

// This value was derived by trial and error.
// Seems to be 1 / 8.
const PIXEL_FONT_TEXEL: f32 = 0.125;

#[derive(Component)]
struct OutlineManager {
    inner_text: Entity,
    outer_texts: [Entity; 4],
}

#[derive(Component)]
#[require(Visibility, Node)]
pub struct TextOutline {
    pub text: String,
    strength: f32,
    pub color: Color,
    outline_color: Color,
    font: TextFont,
    center_text: bool,
}

impl TextOutline {
    pub fn new(
        text: String,
        strength: f32,
        color: Color,
        outline_color: Color,
        font: TextFont,
        center_text: bool,
    ) -> Self {
        Self {
            text,
            strength,
            color,
            outline_color,
            font,
            center_text,
        }
    }
}

fn spawn_text(
    commands: &mut Commands,
    outline: &TextOutline,
    color: Color,
    parent: Entity,
    offset: Vec2,
    zindex: i32,
) -> Entity {
    let justify_content = if outline.center_text {
        JustifyContent::Center
    } else {
        JustifyContent::DEFAULT
    };

    let container = commands
        .spawn((
            ChildOf(parent),
            ZIndex(zindex),
            Node {
                width: Val::Px(0.0),
                height: Val::Px(0.0),
                top: Val::Px(
                    offset.y * outline.font.font_size * outline.strength * PIXEL_FONT_TEXEL,
                ),
                left: Val::Px(
                    offset.x * outline.font.font_size * outline.strength * PIXEL_FONT_TEXEL,
                ),
                justify_content,
                position_type: PositionType::Absolute,
                ..default()
            },
        ))
        .id();

    commands
        .spawn((
            ChildOf(container),
            Node {
                position_type: PositionType::Absolute,
                ..default()
            },
            Text::new(&outline.text),
            TextLayout {
                linebreak: LineBreak::NoWrap,
                ..default()
            },
            outline.font.clone(),
            TextColor(color),
        ))
        .id()
}

fn spawn_outline_text(
    commands: &mut Commands,
    parent: Entity,
    outline: &TextOutline,
    offset: Vec2,
) -> Entity {
    spawn_text(commands, outline, outline.outline_color, parent, offset, 0)
}

fn spawn_inner_text(commands: &mut Commands, parent: Entity, outline: &TextOutline) -> Entity {
    spawn_text(commands, outline, outline.color, parent, Vec2::ZERO, 1)
}

fn spawn_outline_texts(
    mut commands: Commands,
    assets: Res<GameAssets>,
    q_outlines: Query<(Entity, &TextOutline), Added<TextOutline>>,
) {
    for (entity, outline) in &q_outlines {
        debug_assert_eq!(
            outline.font.font, assets.pixel_font,
            "outline only works on pixel font"
        );

        let manager = commands
            .spawn((
                ChildOf(entity),
                Node {
                    width: Val::Px(0.0),
                    height: Val::Px(0.0),
                    position_type: PositionType::Absolute,
                    ..default()
                },
            ))
            .id();

        let inner_text = spawn_inner_text(&mut commands, manager, outline);

        let mut outer_texts = [Entity::PLACEHOLDER; 4];
        for (i, offset) in [Vec2::Y, Vec2::X, Vec2::NEG_Y, Vec2::NEG_X]
            .iter()
            .enumerate()
        {
            outer_texts[i] = spawn_outline_text(&mut commands, manager, outline, *offset);
        }

        commands.entity(manager).insert(OutlineManager {
            inner_text,
            outer_texts,
        });
    }
}

fn update_outline_texts(
    q_outlines: Query<(&Children, &TextOutline), Changed<TextOutline>>,
    q_outline_managers: Query<&OutlineManager>,
    mut q_texts: Query<(&mut Text, &mut TextColor)>,
) {
    for (children, outline) in &q_outlines {
        for child in children {
            let Ok(manager) = q_outline_managers.get(*child) else {
                continue;
            };

            let Ok((mut inner_text, mut inner_color)) = q_texts.get_mut(manager.inner_text) else {
                continue;
            };

            inner_text.0 = outline.text.clone();
            inner_color.0 = outline.color;

            for outer_text_entity in manager.outer_texts {
                let Ok((mut outer_text, _)) = q_texts.get_mut(outer_text_entity) else {
                    continue;
                };
                outer_text.0 = outline.text.clone();
            }
        }
    }
}

pub struct UiOutlinePlugin;

impl Plugin for UiOutlinePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (spawn_outline_texts, update_outline_texts)
                .chain()
                .run_if(resource_exists::<GameAssets>),
        );
    }
}
