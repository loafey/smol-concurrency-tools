
cargo doc --no-deps --features select
rm -rf target/docs
rm -rf ./docs
echo "<meta http-equiv=\"refresh\" content=\"0; url=smol_concurrency_tools\">" > target/doc/index.html
cp -r target/doc ./docs