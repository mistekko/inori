use super::*;
use crate::event_handler::Result;
use crate::model::selector_state::*;
use crate::model::LibActiveSelector::*;
use crate::model::TrackSelItem::*;
use crate::util::song_album;
use mpd::Query;
use mpd::Term;
use std::borrow::Cow::Borrowed;

pub fn handle_library(model: &mut Model, msg: Message) -> Result<()> {
    match msg {
        Message::Search(SearchMsg::Start) => match model.library.active {
            ArtistSelector => {
                model.library.search.active = true;
                model.state = State::Searching;
            }
            TrackSelector => unimplemented!(),
        },
        Message::Search(SearchMsg::End) => model.state = State::Running,
        Message::Escape => model.library.search.active = false,
        Message::Tab => model.screen = Screen::Queue,
        other => match model.library.active {
            ArtistSelector => handle_library_artist(model, other)?,
            TrackSelector => handle_library_track(model, other)?,
        },
    }
    Ok(())
}

pub fn handle_search(model: &mut Model, k: KeyEvent) -> Result<()> {
    match model.library.active {
        LibActiveSelector::ArtistSelector => {
            if let Some(m) = handle_search_k(&mut model.library, k) {
                handle_msg(model, m);
            }
        }
        LibActiveSelector::TrackSelector => unimplemented!(),
    }
    Ok(())
}

pub fn handle_library_artist(model: &mut Model, msg: Message) -> Result<()> {
    match msg {
        Message::Direction(Dirs::Vert(d)) => {
            handle_vertical(d, &mut model.library)
        }
        Message::Direction(Dirs::Horiz(Horizontal::Right)) => {
            model.library.active = TrackSelector;
            model
                .library
                .selected_item_mut()
                .and_then(|f| Some(f.init()));
        }
        Message::Enter => {
            if let Some(artist) = model.library.selected_item() {
                model.conn.findadd(Query::new().and(
                    Term::Tag(Borrowed("AlbumArtist")),
                    artist.name.clone(),
                ))?;
            }
        }
        _ => (),
    }
    Ok(())
}

pub fn add_item(model: &mut Model) -> Result<()> {
    if let Some(artist) = model.library.selected_item_mut() {
        match artist.selected_item() {
            Some(Album(album)) => model.conn.findadd(
                Query::new()
                    .and(
                        Term::Tag(Borrowed("AlbumArtist")),
                        artist.name.clone(),
                    )
                    .and(Term::Tag(Borrowed("Album")), album.name.clone()),
            )?,
            Some(Song(song)) => model
                .conn
                .findadd(Query::new().and(Term::File, song.file.clone()))?,
            None => {}
        }
    }
    Ok(())
}

pub fn handle_library_track(model: &mut Model, msg: Message) -> Result<()> {
    match msg {
        Message::Direction(Dirs::Vert(d)) => {
            if let Some(art) = model.library.selected_item_mut() {
                handle_vertical(d, art)
            }
        }
        Message::Direction(Dirs::Horiz(Horizontal::Left)) => {
            model.library.active = ArtistSelector
        }
        Message::Enter => add_item(model)?,
        Message::Fold | Message::Direction(Dirs::Horiz(Horizontal::Right)) => {
            if let Some(art) = model.library.selected_item_mut() {
                if let Some(album) = art.selected_album_mut() {
                    album.expanded = !album.expanded;
                }
            }
        }
        _ => {}
    }
    Ok(())
}
