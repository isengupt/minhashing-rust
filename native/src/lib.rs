use neon::prelude::*;
extern crate serde_json;

#[macro_use]
extern crate serde_derive;

extern crate crc32fast;

use std::collections::HashSet;
use crc32fast::Hasher;
use itertools::Itertools;
use combinations::Combinations;
use std::iter::Repeat;
use rand::Rng;
use array2d::Array2D;
extern crate optimize;

use optimize::NelderMeadBuilder;
use optimize::Minimizer;


#[macro_use] extern crate itertools;
#[derive(PartialEq, Debug)]
struct Document {
    _id: String,
    text: String,
}

#[derive(PartialEq, Debug)]
struct Result {
    _id1: String,
    _id2: String,
    jaccardSim: f32
}

#[derive(PartialEq, Debug)]
struct HashResult {
    _id1: String,
    _id2: String,
   score: usize
}
#[derive(Debug, Eq, PartialEq)]
struct Shingles {
    _id: String,
    shingles: HashSet<u32>
}
#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
struct Shingle_Items<'a>{
    shingle: &'a u32,
    docs: Vec<String>
}






fn parse_documents(docs: &mut Vec<Document>) {
    for mut doc in docs {
        doc.text = doc.text.trim().replace(&['(', ')','ï','É', 'ô','ë','à', 'â','Á', 'Ú', 'ê','ç', '…', ',', '\"', '.', ';', ':', '\'', '’','‘', '_', '-', '—','“','”', '”', 'ó','é', 'Ó', 'á', '£', '′', 'ú', '•', 'ñ', '\u{ad}', 'í', '–', 'ä', '‐', '¿', '?', 'è', 'Ã', 'ö'][..], "").to_ascii_lowercase().chars().filter(|c| !c.is_whitespace()).collect::<String>();
        doc.text = doc.text.replace(|c: char| !c.is_ascii(), "");
        println!("{:?}", doc.text)
        
    }

}

fn shingle_document(doc: &str, k: i32) -> HashSet<u32>  {
    let mut shingles : HashSet<u32>  = HashSet::new();
    ///let mut string_doc = Vec<char> = doc.text.chars().collect();
    for (i) in (0..doc.len() as i32 - k + 1) {
       // println!("{:?}", i);
        let mut hasher = Hasher::new();
       // println!("{:?}", &doc[i as usize..(i+k) as usize]);
        hasher.update(&doc[i as usize..(i+k) as usize].as_bytes());
        let checksum = hasher.finalize();
        shingles.insert(checksum);
        //println!("{:?}", checksum)
        
    }
    return shingles
}

fn jaccard_similarity(a: HashSet<u32>, b: HashSet<u32>) -> f32 {

let mut union: HashSet<_> = a.union(&b).collect();
let mut intersection: HashSet<_> = a.intersection(&b).collect();


return intersection.len() as f32 / union.len() as f32

}

fn find_duplicates(docs: &mut Vec<Document>, k: i32) {
    let mut results: Vec<Result> = Vec::new();
 
    parse_documents(docs);

   for (i,j) in  iproduct!(docs.iter(), docs.iter()) {

    println!("{:?}", jaccard_similarity(shingle_document(&i.text,k), shingle_document(&j.text,k)));
    results.push(Result { _id1 : i._id.to_string(), _id2: j._id.to_string(), jaccardSim: jaccard_similarity(shingle_document(&i.text,k), shingle_document(&j.text,k)) })
   }

   let filtered_results: Vec<&Result> = results
    .iter()
    .filter(|item| item.jaccardSim > 0.1 && item.jaccardSim < 1.0 )
    .collect();
   println!("{:?}", filtered_results)

}

fn get_shingles(docs: &mut Vec<Document>, k: i32) -> Vec<Shingles> {
    let mut shingle_docs: Vec<Shingles> = Vec::new();
    parse_documents(docs);

    for doc in docs {
        //println!("{:?}", doc);
        shingle_docs.push(Shingles { _id: doc._id.to_string(), shingles: shingle_document(&doc.text.to_string(), k)});
    }

    return shingle_docs
 


}

fn invert_shingles(shingled_docs: &mut Vec<Shingles>) -> (Vec<Shingle_Items>, Vec<String>) {
    let mut items: Vec<&u32> = Vec::new();
    let mut doc_ids: Vec<String> = Vec::new();
    let mut shingle_items: Vec<Shingle_Items> = Vec::new();
    for doc in shingled_docs {
        doc_ids.push(doc._id.to_string());
        for shingle in &doc.shingles {
                if items.contains(&shingle) {
                 let index = shingle_items.iter().position(|r| r.shingle == shingle).unwrap();
                 shingle_items[index].docs.push(doc._id.to_string());


                } else {
                    shingle_items.push(Shingle_Items{shingle: shingle, docs: vec![doc._id.to_string()]});
                    items.push(shingle);
                }

        }
    }
    shingle_items.sort_by(|a, b| b.shingle.cmp(&a.shingle));
    shingle_items.reverse();
    
    return (shingle_items, doc_ids)

 
}

fn make_random_hash() -> Box<dyn Fn(f64) -> f64> {
    let base: u64 = 2;
    let p: u64 = (base.pow(33) - 355) as u64;
    let m: u64 = 4294967295;
    let mut rng = rand::thread_rng();
    let a =  rng.gen_range(0,p - 1) as f64;
    let b = rng.gen_range(0,p -1) as f64;


    Box::new(move |x| ((a * x as f64 * b) % p as f64) % m as f64)

}

fn collect_hash_functions(num_hashes: i32) -> Vec<Box<dyn Fn(f64) -> f64>> {
    let mut functions: Vec<Box<dyn Fn(f64) -> f64>> = Vec::new();
    for i in 0..num_hashes {
        functions.push(make_random_hash())
    }
    return functions

}


fn make_hash_vectors(num_hashes: i32, m: u64) ->  Box<dyn Fn(Vec<f64>) -> f64> {
    let m: f64 = 4294967295.0;
    let hash_functions = collect_hash_functions(num_hashes);

    let _f = move |vec: Vec<f64>| {
        let mut acc: f64= 0.0;
        for i in 0..vec.len() {
            let h = &hash_functions[i];
            acc = acc + h(vec[i]);
        }
        return acc % m
    };

    return Box::new(_f)
 

}


fn get_minhash_signature(shingled_docs: &mut Vec<Shingles>, num_hashes:i32) ->  (Array2D<f64>, Vec<String>) {
    let (inverted_docs, docs_full) = invert_shingles(shingled_docs);
    let num_docs = docs_full.len();
    let mut sigma_matrix = Array2D::filled_with(f64::INFINITY, num_hashes as usize, num_docs);
    let mut sigmatrix: Vec<Vec<f64>> = Vec::with_capacity(num_hashes as usize);

    for _ in 0..num_hashes as usize {
        sigmatrix.push(vec![f64::INFINITY; num_docs]);
    }

    let hash_funcs = collect_hash_functions(num_hashes);

    for item in inverted_docs {
        for doc in item.docs {
           let doc_index = docs_full.iter().position(|r| r == &doc).unwrap();
           for (i, x) in hash_funcs.iter().enumerate() {
               if sigma_matrix[(i, doc_index)] != 0 as f64 {
                    if x(*item.shingle as f64) < sigma_matrix[(i, doc_index)] {
                        sigma_matrix[(i, doc_index)] = x(*item.shingle as f64).into();
                    }
               }
           }
        }
    }

    return (sigma_matrix, docs_full)

    
} 

fn minhash_similarity(_id1: String, _id2: String, sigma_matrix: &Array2D<f64>, doc_ids: &Vec<String>) -> usize {
    let id1_index = doc_ids.iter().position(|r| r == &_id1).unwrap();
    let id2_index = doc_ids.iter().position(|r| r == &_id2).unwrap();
  //  println!("{:?}", id1_index);
   // println!("{:?}", id2_index);

 

    let mut count = 0;

for i in sigma_matrix.column_iter(id1_index) {
    for j in sigma_matrix.column_iter(id2_index) {
        println!("{:?}", (i,j));
       if (i == j) {
           //println!("equal");
           count = count + 1;

       }
    }
}
println!("{:?}", sigma_matrix.column_len());
println!("{:?}", count / sigma_matrix.column_len());
return count / sigma_matrix.column_len()
   
}

fn process_similar_docs(docs:&mut Vec<Document>, k: i32, num_hashes:i32) {
    let mut shingles = get_shingles(docs, k);

    let (mh_matrix, docids) = get_minhash_signature(&mut shingles, num_hashes);
    let mut results: Vec<HashResult> = Vec::new();

    for (i,j) in  iproduct!(docids.iter(), docids.iter()) {
        let mean_sim = minhash_similarity(i.to_string(), j.to_string(), &mh_matrix, &docids);
        // println!("{:?}", jaccard_similarity(shingle_document(&i.text,k), shingle_document(&j.text,k)));
         results.push(HashResult { _id1 : i.to_string(), _id2: j.to_string(), score: mean_sim  });
        }
        println!("{:?}", results)




}

fn debug_array_of_objects(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    // the way to extract first argument out of the function
    let collection_js = cx.argument::<JsArray>(0)?;

    // this part transform the Js Array into Rust Vector
    let collection_rust: Vec<Handle<JsValue>> = collection_js.to_vec(&mut cx)?;

    // we get first item from the collection
    let mut documents: Vec<Document> = Vec::new();
    for item in collection_rust {
        let item_rust = item.downcast::<JsObject>().unwrap();

        let doc_text = item_rust.get(&mut cx, "text")?.to_string(&mut cx)?.value();
        let doc_id = item_rust.get(&mut cx, "_id")?.to_string(&mut cx)?.value();
        //println!("{:?}", doc_text);
       // println!("{:?}", doc_id );
        documents.push(Document {_id: doc_id, text: doc_text});
    }

    //println!("{:?}", documents);
    find_duplicates(&mut documents, 5);



    Ok(cx.undefined())
}


register_module!(mut cx, {
    
    cx.export_function("debugArrayOfObjects", debug_array_of_objects)
});
