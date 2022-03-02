#[macro_export]
macro_rules! handle_these_rooms {
    ($self:expr, $instructions:expr, $( $x:expr ),* ) => {
        $(
            $instructions.append(&mut instruction_generators::handle_rooms(EventArgs::new($x, $self.starting, &$self.ctrlc_handler, $self.cleanup_on_ctrlc, $self.owner.clone(), $self.prefix.clone(), $self.me.clone(), $self.get_tx(), $self.data.room_to_aliases.clone())));
        )*
    };
}


#[macro_export]
macro_rules! sm_println {
    ($self:expr, $t:expr, $( $x:expr ),* ) => {
        $self.get_tx().send(format!($t  
        $(
            , $x
            )*
        )).unwrap();
    };
}

