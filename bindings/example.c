#include <stdio.h>
#include <string.h>
#include <assert.h>
#include "clue.h"

int main(){
    char* code = "global a = 1; print(a); a = 2; print(a);";
    char* out = clue_compile(code);

    assert(strcmp(out,"a = 1;\nprint(a);\na = 2;\nprint(a);\n"));
    printf("%s\n", out);

    clue_free_string(out);
    return 0;
}
