# clangastana

output llvm ast in xml format keeping the tree structure


There are a few number of applications to output clang ast (abstract syntax tree) in xml format. For example, castxml and gccxml. This application yields XML files with keeping ast tree structure in the XML tree structure.

## How to build

clangastana is depending on `libclang` and its [Rust](http://www.rust-lang.org/)-wrapper [clang-rs](https://github.com/KyleMayes/clang-rs). Be ensure that libclang is installed to your environment.

clangastana uses [Cargo](http://crates.io/). So, to build, just execute `cargo build`.

## How to use

Execute `clangastana --help` to show the help information.

```
% cat hello.c
#include <stdio.h>

int main(void) {
  printf("Hello, World.");
  return 0;
}
% clangastana --skip-non-main-file hello.c
<?xml version="1.0" encoding="utf-8"?>
<TranslationUnit display_name="hello.c">
  <FunctionDecl usr="c:@F@main" src="hello.c:3:5:24" linkage="External" type_kind="FunctionPrototype" type_display_name="int (void)" display_name="main()">
    <CompoundStmt src="hello.c:3:16:35">
      <CallExpr src="hello.c:4:3:39" type_kind="Int" type_display_name="int" display_name="printf">
        <UnexposedExpr src="hello.c:4:3:39" type_kind="Pointer" type_display_name="int (*)(const char *, ...)" display_name="printf">
          <DeclRefExpr src="hello.c:4:3:39" type_kind="FunctionPrototype" type_display_name="int (const char *, ...)" display_name="printf" />
        </UnexposedExpr>
        <UnexposedExpr src="hello.c:4:10:46" type_kind="Pointer" type_display_name="const char *">
          <UnexposedExpr src="hello.c:4:10:46" type_kind="Pointer" type_display_name="char *">
            <StringLiteral src="hello.c:4:10:46" type_kind="ConstantArray" type_display_name="char [14]" display_name="&quot;Hello, World.&quot;" />
          </UnexposedExpr>
        </UnexposedExpr>
      </CallExpr>
      <ReturnStmt src="hello.c:5:3:66">
        <IntegerLiteral src="hello.c:5:10:73" type_kind="Int" type_display_name="int" />
      </ReturnStmt>
    </CompoundStmt>
  </FunctionDecl>
</TranslationUnit>
% clangastana --skip-non-main-file -o hello.xml hello.c
% 
```
