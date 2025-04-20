use crate::{content::Content, state::State};

pub fn generate_tree(state: State) -> String {
    // return fetch_contents(&state.contents).join("\n");
    return fetch_contents(&state.contents);
}

fn fetch_contents(c: &Content) -> String {
    // println!("{c:?}");

    let mut result = vec![];

    match c {
        Content::Directory(d) => {
            let mut r = vec![d.name.clone()];
            let length = d.content.len() - 1;
            for (index, i) in d.content.iter().enumerate() {
                // let l = fetch_contents(i);
                // let lines = l.split("\n").collect::<Vec<&str>>();
                // let length = lines.len() - 1;
                // for (index, line) in lines.iter().enumerate() {

                // for (index, line) in fetch_contents(i).split("\n").enumerate() {
                for (inner_index, line) in fetch_contents(i).split("\n").enumerate() {
                    println!("{index} : {length}");
                    let mut prefix = " ";
                    if index == length {
                        if inner_index == 0 {
                            prefix = "└";
                        }
                    } else {
                        if inner_index == 0 {
                            prefix = "├";
                        } else {
                            prefix = "│";
                        }
                    }
                    r.push(format!(" {prefix} {line}"));
                    // r.push(format!(" {} {line}", if (index != 0) && (index == length) { " " } else { "+" }));
                }
                // r.push(format!(" + {}", fetch_contents(i)));
            }
            println!("R : ==\n{}\n==", r.join("\n"));
            result.push(r.join("\n"));
        },
        Content::File(f) => {
            result.push(f.name.clone());
        }
    }

    println!();
    println!("{result:?}");

    result.join("\n")
    // result

    // let mut result = vec![];

    // // ┌─┬─┐
    // // │ │ │
    // // ├─┼─┤
    // // └─┴─┘

    // match c {
    //     Content::Directory(d) => {
    //         let mut r = format!("├ {}\n", d.name.as_str());

    //         for (index, data) in d.content.iter().enumerate() {
    //             let lines = fetch_contents(data.clone());
    //             // for (n, l) in lines.iter().enumerate() {
    //             //    r += format!("{} {l}\n", if n == lines.len() { "└" } else { if n == 0 { "├" } else { "│" } }).as_str();
    //             // }

    //             let length = lines.len();

    //             r += lines
    //                 .into_iter()
    //                 .enumerate()
    //                 .map(
    //                     |(n, l)|
    //                     format!("{}{l}\n", if n == length { "└" } else { if n == 0 { "├" } else { "│" } })
    //                 ).collect::<Vec<String>>()
    //                 .join("\n").as_str();
    //             // println!("{index} : {:?}", fetch_contents(data.clone()));
    //         }

    //         result.push(r);

    //         // println!("{result:?}");
    //         // for (index, data) in d.content.iter().enumerate() {
    //         //     result += fetch_contents(data.clone(), depth + 1).as_str();
    //         // }
    //     },
    //     Content::File(f) => {
    //         result.push(f.name);
    //         // result += format!("{}├ {}\n", "│  ".repeat(depth as usize), f.name.as_str()).as_str();
    //     }
    // }

    // println!("{result:?}");

    // result
}
