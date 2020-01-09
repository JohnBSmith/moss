
use std::rc::Rc;
use typing::{Type,VariableKind,VariableInfo,TypeTable,VARIADIC};

pub struct SymbolTableNode {
    context: Option<usize>,
    variables: Vec<(String,VariableInfo)>,
    local_count: usize,
    context_count: usize
}

impl SymbolTableNode {
    pub fn from_variables_and_context(
        variables: Vec<(String,VariableInfo)>,
        context: Option<usize>
    ) -> Self {
        Self{variables,context,local_count: 0, context_count: 0}
    }
    pub fn count_context(&self) -> usize {
        self.context_count
    }
    pub fn get(&self, id: &str) -> Option<&VariableInfo> {
        for (s,info) in &self.variables {
            if s==id {return Some(info);}
        }
        return None;
    }
    fn get_index(&self, id: &str) -> Option<usize> {
        for (index,(s,_)) in self.variables.iter().enumerate() {
            if s==id {return Some(index);}
        }
        return None;
    }
    fn contains(&self, id: &str) -> bool {
        return self.get(id).is_some();
    }
    fn push_context(&mut self, id: &str, typ: Type) {
        self.variables.push((id.into(),VariableInfo{
            var: true, ty: typ,
            kind: VariableKind::Context(self.context_count),
        }));
        self.context_count +=1;
    }
    pub fn variables(&self) -> &[(String,VariableInfo)] {
        &self.variables
    }
}

pub struct SymbolTable {
    pub list: Vec<SymbolTableNode>,
    pub index: usize
}

impl SymbolTable {
    pub fn new(tab: &TypeTable) -> Self {
        let type_of_print = tab.fn_type(0,VARIADIC,
            vec![tab.type_object()],
            tab.type_unit()
        );

        let type_var: Rc<str> = Rc::from("T");
        let type_of_len = tab.fn_type(1,1,
            vec![tab.list_of(Type::Atom(type_var.clone()))],
            tab.type_int()
        );
        let type_of_len = Type::poly1(type_var,type_of_len);

        let type_of_str = tab.fn_type(1,1,
            vec![tab.type_object()],
            tab.type_string()
        );

        let type_of_list = tab.fn_type(1,1,
            vec![Type::app(vec![tab.type_range(),
                tab.type_int(),
                tab.type_int(),
                tab.type_unit()
            ])],
            tab.list_of(tab.type_int())
        );

        let type_of_iter = tab.fn_type(1,1,
            vec![tab.type_object()],
            tab.type_object()
        );
        
        let type_of_input = tab.fn_type(0,1,
            vec![tab.type_string()],
            tab.type_string()
        );
        
        let type_of_int = tab.fn_type(1,1,
            vec![tab.type_string()],
            tab.type_int()
        );

        let variables: Vec<(String,VariableInfo)> = vec![
            ("print".into(),VariableInfo::global(type_of_print)),
            ("len".into(),VariableInfo::global(type_of_len)),
            ("str".into(),VariableInfo::global(type_of_str)),
            ("list".into(),VariableInfo::global(type_of_list)),
            ("iter".into(),VariableInfo::global(type_of_iter)),
            ("input".into(),VariableInfo::global(type_of_input)),
            ("int".into(),VariableInfo::global(type_of_int))
        ];

        let node = SymbolTableNode{
            context: None, variables,
            local_count: 0, context_count: 0
        };
        let table = SymbolTable{
            index: 0, list: vec![node]
        };
        return table;
    }

    fn get_rec(&mut self, index: usize, key: &str) -> Option<(usize,usize)> {
        let node = &mut self.list[index];
        if let Some(i) = node.get_index(key) {
            return Some((index,i));
        }else if let Some(context) = node.context {
            if let Some(t) = self.get_rec(context,key) {
                let info = &self.list[t.0].variables[t.1].1;
                if let VariableKind::Global = info.kind {
                    return Some(t);
                }
                let typ = info.ty.clone();
                let node = &mut self.list[index];
                node.push_context(key,typ);
                return Some((index,node.variables.len()-1));
            }else{
                return None;
            }
        }else{
            return None;
        }
    }
    
    pub fn get(&mut self, key: &str) -> Option<&VariableInfo> {
        let index = self.index;
        return match self.get_rec(index,key) {
            Some(t) => Some(&self.list[t.0].variables[t.1].1),
            None => None
        };
    }
    pub fn node(&self) -> &SymbolTableNode {
        &self.list[self.index]
    }
    pub fn context_node(&self) -> Option<&SymbolTableNode> {
        let node = &self.list[self.index];
        if let Some(context) = node.context {
            return Some(&self.list[context]);
        }else{
            return None;
        }
    }
    pub fn local_count(&self) -> usize {
        let index = self.index;
        return self.list[index].local_count;
    }
    pub fn print(&self){
        println!("index: {}",self.index);
        for (i,x) in self.list.iter().enumerate() {
            print!("[{}] ",i);
            for (id,_) in &x.variables {
                print!("{}, ",id);
            }
            println!();
        }
        println!();
    }

    pub fn variable_binding(&mut self, global: bool, is_var: bool,
        id: String, typ: Type
    ) {
        let index = self.index;
        let node = &mut self.list[index];
        if node.contains(&id) {
            panic!();
        }
        let kind = if global {
            VariableKind::Global
        }else{
            node.local_count+=1;
            VariableKind::Local(node.local_count-1)
        };
        node.variables.push((id,VariableInfo{
            var: is_var, ty: typ, kind
        }));
    }
}

