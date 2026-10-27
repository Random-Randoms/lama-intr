set -euo pipefail

exe=../target/debug/lama-intr

function test() {
    loc=$1
    name=$2
    nn=$3
    width=$4

    n=$(printf "%0${width}d" $nn)

    if [[ $n == "054" ]]; then 
        echo "$loc$name$n skipped: STA unsupported in lamac"
    fi

    if [[ $n == "074" ]]; then 
        echo "$loc$name$n skipped: very long"
    fi

    if [[ $n == "110" ]]; then
        echo "$loc$name$n skipped: STA unsupported in lamac"
    fi

    if [[ $n == "111" ]]; then
        echo "$loc$name$n skipped: unsopported in lamac"
    fi

    if [[ $n == "803" ]]; then
        echo "$loc$name$n skipped: unsopported in lamac"
    fi

    if [[ -f "../$loc$name$n.lama" ]] && [[ $n != "054" ]] && [[ $n != "074" ]] && [[ $n != "110" ]] && [[ $n != "111" ]] && [[ $n != "803" ]]; then 
        echo "$loc$name$n"

        tmpdir=/tmp/$loc
        raw_input=../$loc$name$n.input
        input=$tmpdir$name$n.input
        src=../$loc$name$n.lama
        expected=$tmpdir$name$n.expected
        actual=$tmpdir$name$n.actual
        raw_bc=$name$n.bc
        bc=$tmpdir$name$n.bc

        # ensure tmpdir
        mkdir -p $tmpdir

        # remove \r from input
        dos2unix -q -n $raw_input $input

        # get expected result
        cat $input | lamac -s $src > $expected

        # get bytecode
        lamac -b $src
        cat $raw_bc > $bc
        rm $raw_bc

        # run bytecode
        cat $input | $exe --file $bc > $actual

        # diff 

        diff -w --color $expected $actual
    fi
}

for n in {1..112}; do
    test Lama/regression/ test $n 3
done
for n in {801..803}; do
    test Lama/regression/ test $n 3
done
# for n in {1..999}; do
#     test Lama/regression_long/deep-expressions/ generated $n 5
# done
