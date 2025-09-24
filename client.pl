
:- consult('facts.pl').

% REGRAS DE CONSULTA

%Busca os documentos por palavra-chave
search_documents(Keyword) :- 
    findall(Score-DocID-Path-Name,
            (token(DocID, Token, Score),
             sub_string(Token, _, _, _, Keyword), % verifica se a palavra é substring dos tokens
             document(DocID, Path, Name)),
            Results),
    sort(Results, SortedResults), % ordena por score
    reverse(SortedResults, FinalResults), % inverte para ter os maiores primeiro
    take(5, FinalResults, Top5), % mostra apenas o top 5
    print_results(Top5).

take(0, _, []) :- !. % se queremos 0 retorna lista vazia
take(_, [], []) :- !. % caso a lista esteja vazia 
take(N, [X|Xs], [X|Ys]) :- 
    N > 0, % verifica se ainda há necessidade de pegar elementos
    % [x|xs] divida a lista em começo(head) X e fim(tail) Xs
    % [X| Ys] adiciona X à lista de resultado
    N1 is N - 1,  % Decrementa o contador
    take(N1, Xs, Ys). % Chama recursivamente para o resto da lista

print_results([]).
print_results([Score-DocID-Path-Name|Rest]) :-
    format('Score: ~w, DocID: ~w, Path: ~w, Name: ~w~n', [Score, DocID, Path, Name]),
    print_results(Rest).


% Encontrar tokens de um documento
tokens_by_document(DocID, Tokens) :-
    findall(Token-Score, token(DocID, Token, Score), Tokens).

% Encontrar tokens com maior TF-IDF em um documento
top_tokens(DocID, N_tokens, TopTokens) :-
    findall(Score-Token, token(DocID, Token, Score), Pairs),
    sort(0, @>=, Pairs, Sorted),
    take(N_tokens, Sorted, TopPairs),
    pairs_values(TopPairs, TopTokens).

take(0, _, []) :- !.
take(_, [], []) :- !.
take(N_tokens, [H|T], [H|Rest]) :-
    N1 is N_tokens - 1,
    take(N1, T, Rest).


