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
    borrow(BorrowName as RegionName)
    
    // This region is **live on entry** to this statement
    //
    // (i.e., some variable with this region in its type is
    //  live on entry)
    live(R1)
    
    // Data with lifetime R1 flows into reference with lifetime R2
    // within this statement. e.g., if you have `let x = y`, you need
    // a `outlives(Y: X)` statement (where Y is region of `y`, etc).
    outlives(R1: R2)  // holds on entry to block; used for types of things that are assigned
    
    // Indicates that this statement overwrites the data that was borrowed
    // by borrow name (e.g., if `&*x` was borrowed, and `x` is reassigned).
    kill(BorrowName)
  }
  statement { }
  goto { B1 B2 }
}
```

### How to run

You have to install souffle. Once you've done that, you would do
something like:

```
> cargo run -- tests/carry-nest/test.txt
```

This will generate `.facts` files in the `tests/carry-nest` directory. Then you
can run `souffle` like so:

```
> souffle regions.dl -F tests/carry-nest/ -D -
```

This will generate a dump with the set of borrows and where they are considered live:

```
---------------
borrowLiveAt
===============
"B_foo"	"B0/1"
"B_foo"	"B0/2"
"B_bar"	"B0/3"
"B_foo"	"B0/3"
"B_foo"	"B0/4"
"B_foo"	"B0/5"
===============
```

You can then inspect the input to see if that meets your
expectations. =)

