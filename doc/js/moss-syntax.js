

function isalpha(s){
  return /^[a-z]+$/i.test(s);
}

function isdigit(s){
  return /^\d+$/.test(s);
}

var keyword = {
  "sub": 0, "end": 0, "begin": 0,
  "if": 0, "then": 0, "elif": 0, "else": 0,
  "while": 0, "do": 0, "for": 0, "in": 0, "is": 0,
  "break": 0, "continue": 0,
  "try": 0, "catch": 0, "global": 0,
  "not": 0, "and": 0, "or": 0, "table": 0, "of": 0,
  "raise": 0, "yield": 0, "return": 0,
  "true": 0, "false": 0, "null": 0,
  "goto": 0, "label": 0, "use": 0,
  "assert": 0, "function": 0
};

function syntax(s){
  var id,s2,st,c;
  s2="";
  var i=0;
  while(i<s.length){
    c=s[i];
    if(isalpha(c) || s[i]=='_'){
      id="";
      while(i<s.length && (isalpha(s[i]) || isdigit(s[i]) || s[i]=='_')){
        id+=s[i];
        i++;
      }
      if(keyword.hasOwnProperty(id)){
        s2+="<span class='keyword'>"+id+"</span>";
      }else{
        s2+=id;
      }
    }else if(isdigit(c)){
      st="";
      while(i<s.length && isdigit(s[i])){
        st+=s[i];
        i++;
      }
      s2+="<span class='number'>"+st+"</span>";
    }else if(c=='"'){
      st="<span class='string'>\"";
      i++;
      while(i<s.length && s[i]!='"'){
        st+=s[i];
        i++;
      }
      i++;
      st+="\"</span>";
      s2+=st;
    }else if(c=="'"){
      st="<span class='string'>'";
      i++;
      while(i<s.length && s[i]!="'"){
        st+=s[i];
        i++;
      }
      i++;
      st+="'</span>";
      s2+=st;
    }else if(c=='#'){
      st="<span class='comment'>";
      while(i<s.length && s[i]!='\n'){
        st+=s[i];
        i++;
      }
      st+="</span>";
      s2+=st;
    }else if(c=='/' && i+1<s.length && s[i+1]=='*'){
      st="<span class='comment'>/*";
      i+=2;
      while(i+1<s.length && s[i]!='*' && s[i+1]!='/'){
        st+=s[i];
        i++;
      }
      i+=2;
      st+="*/</span>";
      s2+=st;
    }else if(c=='&'){
      st="";
      while(i<s.length && s[i]!=';'){
        st+=s[i];
        i++;
      }
      s2+="<span class='symbol'>"+st+";</span>";
      i++;
    }else if(c=='(' || c==')' || c=='[' || c==']' || c=='{' || c=='}'){
      s2+="<span class='bracket'>"+c+"</span>";
      i++;
    }else if(c=='+' || c=='-' || c=='*' || c=='/' || c=='|' ||
      c=='.' || c=='=' || c=='!' || c==':' || c=='%' || c=='^' || c=='$'
    ){
      s2+="<span class='symbol'>"+c+"</span>";
      i++;
    }else{
      s2+=c;
      i++;
    }
  }
  return s2;
}

function main(){
  var a = document.getElementsByClassName("moss");
  for(var i=0; i<a.length; i++){
    a[i].innerHTML = syntax(a[i].innerHTML);
  }
}

window.onload = main;
