use std::sync::mpsc;
use std::thread;

use libfonthelper::{FontEntry, FontsHelper};
use neon::prelude::*;

type Callback = Box<dyn FnOnce(&Channel) + Send>;

struct GetFonts {
  tx: mpsc::Sender<Messages>,
}

enum Messages {
  Callback(Callback),
}

impl GetFonts {
  fn new<'a, C>(cx: &mut C) -> Self
  where
    C: Context<'a>,
  {
    let (tx, rx) = mpsc::channel::<Messages>();
    let channel = cx.channel();

    thread::spawn(move || {
      while let Ok(message) = rx.recv() {
        match message {
          Messages::Callback(f) => {
            f(&channel);
          }
        }
      }
    });

    GetFonts { tx }
  }

  fn send(
    &self,
    callback: impl FnOnce(&Channel) + Send + 'static,
  ) -> Result<(), mpsc::SendError<Messages>> {
    self.tx.send(Messages::Callback(Box::new(callback)))
  }
}

// Methods exposed to JavaScript
impl GetFonts {
  fn js_get_fonts(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let fonts = GetFonts::new(&mut cx);
    let dirs = cx.argument::<JsArray>(0)?;
    let callback = cx.argument::<JsFunction>(1)?.root(&mut cx);

    let arr: Vec<String> = dirs
      .to_vec(&mut cx)
      .unwrap()
      .iter()
      .map(|js_value| {
        js_value
          .downcast_or_throw::<JsString, FunctionContext>(&mut cx)
          .unwrap()
          .value(&mut cx)
      })
      .collect();

    fonts
      .send(move |channel| {
        let helper = FontsHelper::new(&arr);
        let mut map: Vec<FontEntry> = Vec::new();

        for font in helper {
          map.push(font);
        }

        channel.send(move |mut cx| {
          let callback = callback.into_inner(&mut cx);
          let this = cx.undefined();
          let js_fonts = JsObject::new(&mut cx);

          for font in map {
            if font.path.eq("") {
              continue;
            }

            let js_font_entries = JsArray::new(&mut cx, font.entries.len() as u32);

            for (index, entry) in font.entries.iter().enumerate() {
              let js_font_entry = JsObject::new(&mut cx);

              let id = cx.string(&entry.id);
              let postscript = cx.string(&entry.postscript);
              let family = cx.string(&entry.family);
              let style = cx.string(&entry.style);
              let weight = cx.number(i32::from(entry.weight));
              let stretch = cx.number(i32::from(entry.stretch));
              let italic = cx.boolean(entry.italic);

              js_font_entry.set(&mut cx, "id", id)?;
              js_font_entry.set(&mut cx, "postscript", postscript)?;
              js_font_entry.set(&mut cx, "family", family)?;
              js_font_entry.set(&mut cx, "style", style)?;
              js_font_entry.set(&mut cx, "weight", weight)?;
              js_font_entry.set(&mut cx, "stretch", stretch)?;
              js_font_entry.set(&mut cx, "italic", italic)?;

              js_font_entries.set(&mut cx, index.to_string().as_str(), js_font_entry)?;
            }

            js_fonts.set(&mut cx, font.path.as_str(), js_font_entries)?;
          }

          let args: Vec<Handle<JsObject>> = vec![js_fonts];

          callback.call(&mut cx, this, args)?;

          Ok(())
        });
      })
      .or_else(|err| cx.throw_error(err.to_string()))?;

    Ok(cx.undefined())
  }
}

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
  cx.export_function("getFonts", GetFonts::js_get_fonts)?;

  Ok(())
}
