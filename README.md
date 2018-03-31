This is a wild-and-crazy new formulation of the NLL analysis. It aims
to accept more programs and be more efficiently implementable. This
prototype works by accepting very special input files and generating
"facts" files.  These files can then be interpreted by the datalog
formulation using [souffle](https://github.com/souffle-lang/souffle).

### Input file format

```
// Maybe some comments here

block B1 {
  statement { 
    // A `&foo` statement occured here
    borrow(R1)
    
    // This region is **live on entry** to this statement
    //
    // (i.e., some variable with this region in its type is
    //  live on entry)
    live(R1)
    
    // Data with lifetime R1 flows into reference with lifetime R2
    // within this statement. e.g., `let x = y` would make
    // data with region of Y flow into region. of X
    R1: R2  // holds on entry to block; used for types of things that are assigned
  }
  statement { }
  goto { B1 B2 }
}

let 
```
