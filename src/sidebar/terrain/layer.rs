use crate::{
    gui::{BuildContext, EditorUiNode, Ui, UiMessage, UiNode},
    make_relative_path,
    scene::commands::{
        terrain::{SetTerrainLayerTextureCommand, TerrainLayerTextureKind},
        SceneCommand,
    },
    send_sync_message,
    sidebar::{make_text_mark, COLUMN_WIDTH, ROW_HEIGHT},
    Message,
};
use rg3d::gui::Thickness;
use rg3d::{
    core::{pool::Handle, scope_profile},
    engine::resource_manager::ResourceManager,
    gui::{
        grid::{Column, GridBuilder, Row},
        image::ImageBuilder,
        message::{ImageMessage, MessageDirection, UiMessageData, WidgetMessage},
        widget::WidgetBuilder,
    },
    resource::texture::Texture,
    scene::{node::Node, terrain::Layer},
    utils::into_gui_texture,
};
use std::sync::mpsc::Sender;

pub struct LayerSection {
    pub section: Handle<UiNode>,
    diffuse_texture: Handle<UiNode>,
    normal_texture: Handle<UiNode>,
    specular_texture: Handle<UiNode>,
    roughness_texture: Handle<UiNode>,
    height_texture: Handle<UiNode>,
}

fn make_texture_field(ctx: &mut BuildContext, row: usize) -> Handle<UiNode> {
    ImageBuilder::new(
        WidgetBuilder::new()
            .on_column(1)
            .on_row(row)
            .with_allow_drop(true)
            .with_margin(Thickness::uniform(1.0)),
    )
    .build(ctx)
}

fn send_image_sync_message(ui: &Ui, image: Handle<UiNode>, texture: Option<Texture>) {
    send_sync_message(
        ui,
        ImageMessage::texture(
            image,
            MessageDirection::ToWidget,
            texture.map(|t| into_gui_texture(t)),
        ),
    );
}

impl LayerSection {
    pub fn new(ctx: &mut BuildContext) -> Self {
        let diffuse_texture;
        let normal_texture;
        let specular_texture;
        let roughness_texture;
        let height_texture;
        let section = GridBuilder::new(
            WidgetBuilder::new()
                .with_child(make_text_mark(ctx, "Diffuse Texture", 0))
                .with_child({
                    diffuse_texture = make_texture_field(ctx, 0);
                    diffuse_texture
                })
                .with_child(make_text_mark(ctx, "Normal Texture", 1))
                .with_child({
                    normal_texture = make_texture_field(ctx, 1);
                    normal_texture
                })
                .with_child(make_text_mark(ctx, "Specular Texture", 2))
                .with_child({
                    specular_texture = make_texture_field(ctx, 2);
                    specular_texture
                })
                .with_child(make_text_mark(ctx, "Roughness Texture", 3))
                .with_child({
                    roughness_texture = make_texture_field(ctx, 3);
                    roughness_texture
                })
                .with_child(make_text_mark(ctx, "Height Texture", 4))
                .with_child({
                    height_texture = make_texture_field(ctx, 4);
                    height_texture
                }),
        )
        .add_column(Column::strict(COLUMN_WIDTH))
        .add_column(Column::stretch())
        .add_row(Row::strict(ROW_HEIGHT))
        .add_row(Row::strict(ROW_HEIGHT))
        .add_row(Row::strict(ROW_HEIGHT))
        .add_row(Row::strict(ROW_HEIGHT))
        .add_row(Row::strict(ROW_HEIGHT))
        .build(ctx);

        Self {
            section,
            diffuse_texture,
            normal_texture,
            specular_texture,
            roughness_texture,
            height_texture,
        }
    }

    pub fn sync_to_model(&mut self, layer: Option<&Layer>, ui: &mut Ui) {
        send_sync_message(
            ui,
            WidgetMessage::visibility(self.section, MessageDirection::ToWidget, layer.is_some()),
        );

        if let Some(layer) = layer {
            send_image_sync_message(ui, self.diffuse_texture, layer.diffuse_texture.clone());
            send_image_sync_message(ui, self.normal_texture, layer.normal_texture.clone());
            send_image_sync_message(ui, self.specular_texture, layer.specular_texture.clone());
            send_image_sync_message(ui, self.roughness_texture, layer.roughness_texture.clone());
            send_image_sync_message(ui, self.height_texture, layer.height_texture.clone());
        }
    }

    pub fn handle_message(
        &mut self,
        message: &UiMessage,
        ui: &mut Ui,
        index: usize,
        resource_manager: ResourceManager,
        node_handle: Handle<Node>,
        sender: &Sender<Message>,
    ) {
        scope_profile!();

        if let UiMessageData::Widget(WidgetMessage::Drop(handle)) = *message.data() {
            if let UiNode::User(EditorUiNode::AssetItem(item)) = ui.node(handle) {
                let relative_path = make_relative_path(&item.path);

                let kind_field = if message.destination() == self.diffuse_texture {
                    Some((TerrainLayerTextureKind::Diffuse, self.diffuse_texture))
                } else if message.destination() == self.normal_texture {
                    Some((TerrainLayerTextureKind::Normal, self.normal_texture))
                } else if message.destination() == self.specular_texture {
                    Some((TerrainLayerTextureKind::Specular, self.specular_texture))
                } else if message.destination() == self.roughness_texture {
                    Some((TerrainLayerTextureKind::Roughness, self.roughness_texture))
                } else if message.destination() == self.height_texture {
                    Some((TerrainLayerTextureKind::Height, self.height_texture))
                } else {
                    None
                };

                if let Some((kind, field)) = kind_field {
                    let texture = resource_manager.request_texture(relative_path);

                    sender
                        .send(Message::DoSceneCommand(
                            SceneCommand::SetTerrainLayerTexture(
                                SetTerrainLayerTextureCommand::new(
                                    node_handle,
                                    index,
                                    texture.clone(),
                                    kind,
                                ),
                            ),
                        ))
                        .unwrap();

                    ui.send_message(ImageMessage::texture(
                        field,
                        MessageDirection::ToWidget,
                        Some(into_gui_texture(texture)),
                    ));
                }
            }
        }
    }
}