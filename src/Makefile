CC      = gcc
PROF    = -g
NOCRYPT =
C_FLAGS = -O -Wall $(PROF) $(NOCRYPT) -U_FORTIFY_SOURCE -D_FORTIFY_SOURCE=0 -fno-stack-protector
L_FLAGS = -O $(PROF)
LIBS   = -lcrypt -lm

O_FILES = act_comm.o act_info.o act_move.o act_obj.o act_wiz.o bit.o board.o clan.o  \
          comm.o const.o db.o envconfig.o fight.o fist.o handler.o interp.o mazoku.o \
	  magic.o mem.o mob_commands.o mob_prog.o olc.o olc_act.o olc_save.o patryn.o \
	  quest.o saiyan.o save.o special.o soldier.o sorcerer.o string.o suit.o     \
	  update.o 

# --- Rust integration (optional, activated with USE_RUST=1) ---
ifdef USE_RUST
RUST_LIB_PATH = ../target/release/libchaos_rust.a
O_FILES := $(filter-out envconfig.o,$(O_FILES))
RUST_LIBS = -ldl -lpthread -lm -lgcc_s
RUST_DEP = rust_lib

.PHONY: rust_lib
rust_lib:
	$(HOME)/.cargo/bin/cargo build --release --manifest-path ../chaos-rust/Cargo.toml
endif
# --- End Rust integration ---

chaosium: $(O_FILES) $(RUST_DEP)
	rm -f chaosium 
ifdef USE_RUST
	$(CC) $(L_FLAGS) -o chaosium $(O_FILES) $(RUST_LIB_PATH) $(LIBS) $(RUST_LIBS)
else
	$(CC) $(L_FLAGS) -o chaosium $(O_FILES) $(LIBS)
endif

.c.o: merc.h
	$(CC) -c $(C_FLAGS) $<

clean:
	rm -f *.o chaosium
